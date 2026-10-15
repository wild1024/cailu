import service, { ApiError, del, get, type ApiResult, type PageData } from '@/api/request';
import type { StatusValue } from '@/api/system/user-api';

/** 文件记录 */
export interface FileRes {
  /** 文件 id */
  id: string;
  /** 存储文件名 */
  file_name: string;
  /** 原始文件名 */
  original_name: string;
  /** 存储 key */
  file_key: string;
  /** 文件大小（字节） */
  file_size: number;
  /** MIME 类型 */
  mime_type: string;
  /** 扩展名 */
  extension: string;
  /** 文件分类 */
  category: string;
  /** 状态 */
  status: StatusValue;
  /** 上传人 */
  uploaded_by: string;
  /** 上传时间 */
  created_at: string;
  /** 访问地址 */
  url: string;
}

/** 文件分页查询参数 */
export type FilePageQuery = {
  /** 页码，从 1 开始 */
  page: number;
  /** 每页条数 */
  page_size: number;
  /** 原始文件名（模糊） */
  original_name?: string;
  /** 文件分类 */
  category?: string;
  /** MIME 类型 */
  mime_type?: string;
};

/** 上传文件（multipart，字段名固定为 file） */
export function uploadFile(file: File, category?: string): Promise<FileRes> {
  const formData = new FormData();
  formData.append('file', file);
  const query = category ? `?category=${encodeURIComponent(category)}` : '';
  return service
    .post<ApiResult<FileRes>>(`/api/support/file/upload${query}`, formData)
    .then((response) => {
      const body = response.data;
      if (!body || typeof body.code !== 'number' || body.code !== 0) {
        throw new ApiError(typeof body?.code === 'number' ? body.code : -1, body?.msg || '上传失败');
      }
      return body.data;
    });
}

/** 分页查询文件 */
export function pageFiles(query: FilePageQuery): Promise<PageData<FileRes>> {
  return get<PageData<FileRes>>('/api/support/file/page', query);
}

/** 查询文件详情 */
export function getFile(id: string): Promise<FileRes> {
  return get<FileRes>(`/api/support/file/${id}`);
}

/** 删除文件 */
export function deleteFile(id: string): Promise<null> {
  return del<null>(`/api/support/file/${id}`);
}

/** 下载文件，返回二进制内容（携带鉴权头，由调用方触发浏览器保存） */
export async function downloadFileBlob(id: string): Promise<Blob> {
  const response = await service.get<Blob>(`/api/support/file/download/${id}`, {
    responseType: 'blob',
  });
  return response.data;
}

/** 文件模块接口集合 */
export const fileApi = {
  uploadFile,
  pageFiles,
  getFile,
  deleteFile,
  downloadFileBlob,
};
