import dayjs from 'dayjs';

/** 统一的时间展示格式 */
const DATE_TIME_FORMAT = 'YYYY-MM-DD HH:mm:ss';

/**
 * 格式化后端返回的时间字符串。
 *
 * 空值与非法时间统一展示为 `-`，避免界面出现 `Invalid Date`。
 */
export function formatDateTime(value?: string | null): string {
  if (!value) {
    return '-';
  }
  const parsed = dayjs(value);
  return parsed.isValid() ? parsed.format(DATE_TIME_FORMAT) : '-';
}

/** 格式化文件大小（字节）为可读文本 */
export function formatFileSize(size?: number | null): string {
  if (size === undefined || size === null || size < 0) {
    return '-';
  }
  if (size < 1024) {
    return `${size} B`;
  }
  const units = ['KB', 'MB', 'GB', 'TB'];
  let value = size / 1024;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${value.toFixed(2)} ${units[index]}`;
}