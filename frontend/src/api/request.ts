import axios, { AxiosError, type AxiosInstance, type AxiosRequestConfig } from 'axios';

/** 后端统一响应体（对应 Rust 侧 `Res<T>`） */
export interface ApiResult<T = unknown> {
  /** 业务状态码，0 表示成功 */
  code: number;
  /** 提示消息 */
  msg: string;
  /** 业务数据 */
  data: T;
}

/** 分页数据体（对应后端 `Res::page` 的 data 字段） */
export interface PageData<T> {
  /** 当前页数据 */
  list: T[];
  /** 总记录数 */
  total: number;
  /** 当前页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
}

/** 访问令牌在 localStorage 中的键名 */
export const TOKEN_KEY = 'x_access_token';
/** 刷新令牌在 localStorage 中的键名 */
export const REFRESH_TOKEN_KEY = 'x_refresh_token';

/** 接口基础路径，与 axios 实例保持一致 */
const BASE_URL = import.meta.env.VITE_APP_BASE_API || '';

/** 刷新令牌接口路径（免登录，401 时自动调用） */
const REFRESH_PATH = '/api/system/auth/refresh';

/** 登录页路径，令牌彻底失效后跳转至此 */
const LOGIN_PATH = `${import.meta.env.VITE_APP_CONTEXT_PATH || '/'}login`.replace(/\/{2,}/g, '/');

/** 刷新令牌接口的返回体（仅取本模块所需字段，避免与 api/auth 循环依赖） */
interface RefreshPayload {
  /** 新的访问令牌 */
  access_token: string;
  /** 新的刷新令牌 */
  refresh_token: string;
}

/** 支持「401 后重放一次」的请求配置，标记位用于防止无限重试 */
type RetriableConfig = AxiosRequestConfig & { /** 是否已经因 401 重放过 */ _retried?: boolean };

/**
 * 业务异常：后端返回了 `code !== 0` 的统一响应体，或 HTTP 层请求失败。
 *
 * 调用方通过 `instanceof ApiError` 区分「业务失败」与「代码缺陷」。
 */
export class ApiError extends Error {
  /** 业务状态码；网络层失败时为 0 */
  readonly code: number;

  constructor(code: number, message: string) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
  }
}

const service: AxiosInstance = axios.create({
  baseURL: BASE_URL,
  timeout: 15000,
});

// 请求拦截：注入 Bearer Token，避免每个调用点重复处理鉴权头
service.interceptors.request.use((config) => {
  const token = localStorage.getItem(TOKEN_KEY);
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 响应拦截：401 时用 refresh_token 换新令牌并重放原请求，其余错误规整成 ApiError
service.interceptors.response.use(
  (response) => response,
  async (error: AxiosError<ApiResult>) => {
    const config = error.config as RetriableConfig | undefined;
    if (shouldRefresh(error, config)) {
      // 标记已重放，避免刷新成功但接口仍返回 401 时陷入死循环
      config._retried = true;
      const accessToken = await refreshAccessToken();
      if (accessToken) {
        const headers = config.headers ?? {};
        headers.Authorization = `Bearer ${accessToken}`;
        config.headers = headers;
        return service.request(config);
      }
      redirectToLogin();
    }
    return Promise.reject(toApiError(error));
  },
);

/** 判断当前失败是否需要走「刷新令牌 + 重放」流程 */
function shouldRefresh(error: AxiosError<ApiResult>, config?: RetriableConfig): config is RetriableConfig {
  if (error.response?.status !== 401 || !config || config._retried) {
    return false;
  }
  // 刷新接口自身返回 401 时不再递归刷新
  return config.url !== REFRESH_PATH;
}

/** 进行中的刷新请求，用于把并发 401 合并成一次刷新 */
let refreshing: Promise<string | null> | null = null;

/**
 * 用本地 refresh_token 换取新的访问令牌。
 *
 * # 返回值
 *
 * 刷新成功返回新的 access_token；无 refresh_token 或刷新失败返回 null（并清空本地令牌）。
 */
function refreshAccessToken(): Promise<string | null> {
  refreshing ??= doRefresh().finally(() => {
    refreshing = null;
  });
  return refreshing;
}

/** 真正执行刷新请求；此处用裸 axios，避免复用带拦截器的实例造成递归 */
async function doRefresh(): Promise<string | null> {
  const refreshToken = localStorage.getItem(REFRESH_TOKEN_KEY);
  if (!refreshToken) {
    clearTokens();
    return null;
  }
  try {
    const response = await axios.post<ApiResult<RefreshPayload>>(
      REFRESH_PATH,
      { refresh_token: refreshToken },
      { baseURL: BASE_URL, timeout: 15000 },
    );
    const body = response.data;
    if (!body || body.code !== 0 || !body.data?.access_token) {
      clearTokens();
      return null;
    }
    saveTokens(body.data.access_token, body.data.refresh_token);
    return body.data.access_token;
  } catch {
    clearTokens();
    return null;
  }
}

/** 令牌彻底失效时跳转登录页（replace 避免把失效页面留在历史记录里） */
function redirectToLogin(): void {
  if (window.location.pathname.endsWith('/login')) {
    return;
  }
  window.location.replace(LOGIN_PATH);
}

/** 把 axios 的各类失败（网络、超时、非 2xx）转换为带可读消息的 ApiError */
function toApiError(error: AxiosError<ApiResult>): ApiError {
  const body = error.response?.data;
  const status = error.response?.status;
  // 后端在 401/403/500 等错误路径上也返回统一响应体，优先采用其 msg
  if (body && typeof body === 'object' && typeof body.msg === 'string' && body.msg) {
    if (status === 401) {
      clearTokens();
    }
    return new ApiError(typeof body.code === 'number' ? body.code : -1, body.msg);
  }
  if (status === 401) {
    clearTokens();
    return new ApiError(-1, '登录状态已失效，请重新登录');
  }
  if (status === 403) {
    return new ApiError(-1, '权限不足');
  }
  if (error.code === 'ECONNABORTED') {
    return new ApiError(-1, '请求超时，请稍后重试');
  }
  return new ApiError(-1, error.message || '网络异常，请检查服务是否可用');
}

/** 清除本地令牌（登出或令牌失效时调用） */
export function clearTokens(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(REFRESH_TOKEN_KEY);
}

/** 保存令牌对 */
export function saveTokens(accessToken: string, refreshToken?: string): void {
  localStorage.setItem(TOKEN_KEY, accessToken);
  if (refreshToken) {
    localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken);
  }
}

/**
 * 发起请求并拆解统一响应体。
 *
 * # 返回值
 *
 * 成功时返回后端 `data` 字段的内容；`code !== 0` 或 HTTP 失败时抛出 {@link ApiError}。
 */
async function request<T>(config: AxiosRequestConfig): Promise<T> {
  const response = await service.request<ApiResult<T>>(config);
  const body = response.data;
  if (body === null || typeof body !== 'object' || typeof body.code !== 'number') {
    throw new ApiError(-1, '响应体格式不符合统一响应规范');
  }
  if (body.code !== 0) {
    throw new ApiError(body.code, body.msg);
  }
  return body.data;
}

/** GET 请求 */
export function get<T>(url: string, params?: Record<string, unknown>): Promise<T> {
  return request<T>({ url, method: 'GET', params });
}

/** POST 请求 */
export function post<T>(url: string, data?: unknown): Promise<T> {
  return request<T>({ url, method: 'POST', data });
}

/** PUT 请求 */
export function put<T>(url: string, data?: unknown): Promise<T> {
  return request<T>({ url, method: 'PUT', data });
}

/** PATCH 请求（部分更新） */
export function patch<T>(url: string, data?: unknown): Promise<T> {
  return request<T>({ url, method: 'PATCH', data });
}

/** DELETE 请求 */
export function del<T>(url: string, params?: Record<string, unknown>): Promise<T> {
  return request<T>({ url, method: 'DELETE', params });
}

export default service;
