// 发给后端前统一处理
export function toBackendNumber(n: bigint | string | number): string | number {
  if (typeof n === "bigint") {
    // 任何 bigint 一律转 string
    return n.toString();
  }
  if (typeof n === "string") {
    // 如果是纯数字且 < 2^53，直接传 number 也行（节省流量）
    if (/^\d+$/.test(n) && BigInt(n) < 2n ** 53n) {
      return Number(n);
    }
    return n; // 否则走 string
  }
  if (typeof n === "number" && Number.isSafeInteger(n)) {
    return n;
  }
  // 其他情况强转 bigint 再转 string
  return BigInt(Math.floor(n)).toString();
}

// 接收后端数据后转成统一类型
export function fromBackendNumber(n: string | number): bigint {
  return typeof n === "string" ? BigInt(n) : BigInt(n);
}