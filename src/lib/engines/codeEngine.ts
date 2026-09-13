import type { PreprocessedResult } from "@/types";

/**
 * 1. JSON 格式化与校验引擎
 */
export function formatJson(input: string, minify: boolean = false): string {
  const trimmed = input.trim();
  if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) {
    throw new Error("无效的 JSON 格式 (内容必须以 { 或 [ 开头)");
  }
  try {
    const parsed = JSON.parse(trimmed);
    if (minify) {
      return JSON.stringify(parsed);
    }
    return JSON.stringify(parsed, null, 2);
  } catch (err: any) {
    throw new Error(`JSON 解析失败: ${err?.message || "语法错误"}`);
  }
}

/**
 * 2. URL 编解码与 Query 参数结构化解析
 */
export function parseUrl(input: string): string {
  const trimmed = input.trim();
  if (trimmed.length > 2048) {
    throw new Error("输入的 URL 长度超出常规限制");
  }
  try {
    // If not starts with protocol, attempt standard http prepend for parsing
    let parseTarget = trimmed;
    if (!trimmed.includes("://") && !trimmed.startsWith("/")) {
      parseTarget = "http://" + trimmed;
    }

    const url = new URL(parseTarget);
    const queryParams: Record<string, string | string[]> = {};

    url.searchParams.forEach((val, key) => {
      if (queryParams[key]) {
        if (Array.isArray(queryParams[key])) {
          (queryParams[key] as string[]).push(val);
        } else {
          queryParams[key] = [queryParams[key] as string, val];
        }
      } else {
        queryParams[key] = val;
      }
    });

    const result = {
      original: trimmed.length > 120 ? `${trimmed.slice(0, 120)}...` : trimmed,
      decodedUrl: decodeURI(trimmed),
      protocol: url.protocol,
      hostname: url.hostname,
      port: url.port || (url.protocol === "https:" ? "443" : "80"),
      pathname: url.pathname,
      queryParams: Object.keys(queryParams).length > 0 ? queryParams : undefined,
      hash: url.hash || undefined,
    };

    return JSON.stringify(result, null, 2);
  } catch {
    // If not a full URL, attempt standard decodeURIComponent
    try {
      const decoded = decodeURIComponent(trimmed);
      return JSON.stringify(
        { decoded, original: trimmed.length > 120 ? `${trimmed.slice(0, 120)}...` : trimmed },
        null,
        2,
      );
    } catch {
      throw new Error("无效的 URL 格式");
    }
  }
}

/**
 * 3. JWT 结构化解析器
 */
export function inspectJwt(token: string): string {
  const trimmed = token.trim();
  const parts = trimmed.split(".");
  if (parts.length !== 3) {
    throw new Error("无效的 JWT 格式 (必须由两点分隔的三段组成)");
  }

  function b64urlDecode(str: string): any {
    try {
      let base64 = str.replace(/-/g, "+").replace(/_/g, "/");
      while (base64.length % 4) {
        base64 += "=";
      }
      const json = atob(base64);
      return JSON.parse(decodeURIComponent(escape(json)));
    } catch {
      try {
        let base64 = str.replace(/-/g, "+").replace(/_/g, "/");
        while (base64.length % 4) {
          base64 += "=";
        }
        return JSON.parse(atob(base64));
      } catch (e: any) {
        return { error: `解码失败: ${e?.message || e}` };
      }
    }
  }

  const header = b64urlDecode(parts[0]);
  const payload = b64urlDecode(parts[1]);

  // Transform timestamps to human readable
  const enrichedPayload = { ...payload };
  const nowSec = Math.floor(Date.now() / 1000);

  if (typeof payload.exp === "number") {
    enrichedPayload["_exp_formatted"] = new Date(payload.exp * 1000).toLocaleString();
    enrichedPayload["_is_expired"] = payload.exp < nowSec;
  }
  if (typeof payload.iat === "number") {
    enrichedPayload["_iat_formatted"] = new Date(payload.iat * 1000).toLocaleString();
  }
  if (typeof payload.nbf === "number") {
    enrichedPayload["_nbf_formatted"] = new Date(payload.nbf * 1000).toLocaleString();
  }

  const result = {
    header,
    payload: enrichedPayload,
    signature: parts[2],
  };

  return JSON.stringify(result, null, 2);
}

// 格式化精确值：大于等于1最多保留3位小数，小于1最多保留7位小数，均去除末尾的0
export function formatExactNumber(val: number): string {
  if (isNaN(val) || !isFinite(val)) return String(val);
  const abs = Math.abs(val);
  let s = abs >= 1 ? val.toFixed(3) : val.toFixed(7);
  if (s.includes(".")) s = s.replace(/\.?0+$/, "");
  return s;
}

// 格式化英语习惯：以 K M G T P 等为单位，保留最多3位小数，去除末尾的0
export function formatEnglishNumber(val: number): string {
  if (isNaN(val) || !isFinite(val)) return String(val);
  const sign = val < 0 ? "-" : "";
  const abs = Math.abs(val);
  const fmt = (n: number, u: string) => `${sign}${n.toFixed(3).replace(/\.?0+$/, "")}${u}`;

  if (abs >= 1e15) return fmt(abs / 1e15, "P");
  if (abs >= 1e12) return fmt(abs / 1e12, "T");
  if (abs >= 1e9) return fmt(abs / 1e9, "G");
  if (abs >= 1e6) return fmt(abs / 1e6, "M");
  if (abs >= 1e3) return fmt(abs / 1e3, "K");
  return formatExactNumber(val);
}

// 格式化中文习惯：以 千/万/亿/万亿 等为单位，保留最多3位小数，去除末尾的0
export function formatChineseNumber(val: number): string {
  if (isNaN(val) || !isFinite(val)) return String(val);
  const sign = val < 0 ? "-" : "";
  const abs = Math.abs(val);
  const fmt = (n: number, u: string) => `${sign}${n.toFixed(3).replace(/\.?0+$/, "")}${u}`;

  if (abs >= 1e12) return fmt(abs / 1e12, "万亿");
  if (abs >= 1e8) return fmt(abs / 1e8, "亿");
  if (abs >= 1e4) return fmt(abs / 1e4, "万");
  if (abs >= 1e3) return fmt(abs / 1e3, "千");
  return formatExactNumber(val);
}

/**
 * 4. 时间戳双向转换器
 */
export function convertTimestamp(input: string): string {
  const trimmed = input.trim();

  // Fast-fail if input is excessively long or has image/base64 characteristics
  if (
    trimmed.length > 100 ||
    trimmed.startsWith("data:") ||
    trimmed.startsWith("{") ||
    trimmed.startsWith("[")
  ) {
    throw new Error("无法识别的时间戳格式 (输入内容不符合时间戳或日期特征)");
  }

  let sec: number;
  let ms: number;
  let pythonFloat: string;
  let subsecondsStr: string | undefined;

  // 1. Python 浮点数时间戳 (如 1789264888.123456, 1789264888.0)
  const isFloatTimestamp = /^\d{9,11}\.\d+$/.test(trimmed);
  let isSingleNumber = false;
  let singleNumVal = 0;

  if (isFloatTimestamp) {
    const floatVal = parseFloat(trimmed);
    if (isNaN(floatVal) || floatVal <= 0 || floatVal > 253402300799) {
      throw new Error("无效的浮点时间戳数值");
    }
    sec = Math.floor(floatVal);
    ms = Math.round(floatVal * 1000);
    pythonFloat = trimmed;
    const parts = trimmed.split(".");
    subsecondsStr = parts[1];
    isSingleNumber = true;
    singleNumVal = floatVal;
  } else {
    const num = Number(trimmed);
    if (!isNaN(num) && /^\d+$/.test(trimmed) && trimmed.length >= 9 && trimmed.length <= 19) {
      // 2. 纯数字整数时间戳 (<=11位秒 / 12~14位毫秒 / 15~17位微秒)
      if (trimmed.length <= 11) {
        sec = num;
        ms = num * 1000;
        pythonFloat = `${sec}.0`;
      } else if (trimmed.length <= 14) {
        sec = Math.floor(num / 1000);
        ms = num;
        const sub = num % 1000;
        pythonFloat = sub === 0 ? `${sec}.0` : (num / 1000).toFixed(3);
        if (sub !== 0) {
          subsecondsStr = String(sub).padStart(3, "0");
        }
      } else if (trimmed.length <= 17) {
        sec = Math.floor(num / 1_000_000);
        ms = Math.floor(num / 1_000);
        const sub = num % 1_000_000;
        pythonFloat = sub === 0 ? `${sec}.0` : (num / 1_000_000).toFixed(6);
        if (sub !== 0) {
          subsecondsStr = String(sub).padStart(6, "0");
        }
      } else {
        sec = Math.floor(num / 1000);
        ms = num;
        pythonFloat = (num / 1000).toFixed(3);
      }
      isSingleNumber = true;
      singleNumVal = num;
    } else {
      // 3. 日期与时间字符串 (如 2026-09-13 10:01:28+08:00, 2024-1-2, 2024/01/02 12:00:00, ISO8601)
      let parsedDate = new Date(trimmed);
      if (isNaN(parsedDate.getTime())) {
        parsedDate = new Date(trimmed.replace(/\//g, "-"));
      }
      if (isNaN(parsedDate.getTime())) {
        parsedDate = new Date(trimmed.replace(/\//g, "-").replace(" ", "T"));
      }
      if (isNaN(parsedDate.getTime())) {
        throw new Error("无法识别的时间戳或日期格式");
      }
      ms = parsedDate.getTime();
      sec = Math.floor(ms / 1000);
      const sub = ms % 1000;
      pythonFloat = sub === 0 ? `${sec}.0` : (ms / 1000).toFixed(3);
      if (sub !== 0) {
        subsecondsStr = String(sub).padStart(3, "0");
      }
    }
  }

  const date = new Date(ms);
  if (isNaN(date.getTime())) {
    throw new Error("无效的时间戳数值");
  }

  const pad = (n: number) => String(n).padStart(2, "0");
  const localBase = `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
  const localFormatted = subsecondsStr ? `${localBase}.${subsecondsStr}` : localBase;

  const utcBase = `${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())} ${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}:${pad(date.getUTCSeconds())}`;
  const utcFormatted = subsecondsStr ? `${utcBase}.${subsecondsStr} UTC` : `${utcBase} UTC`;

  const isoFormatted = date.toISOString();

  const now = Date.now();
  const diffSec = Math.floor((now - ms) / 1000);
  let relative = "";
  if (Math.abs(diffSec) < 5) {
    relative = "刚刚";
  } else if (diffSec >= 0) {
    if (diffSec < 60) relative = `${diffSec} 秒前`;
    else if (diffSec < 3600) relative = `${Math.floor(diffSec / 60)} 分钟前`;
    else if (diffSec < 86400) relative = `${Math.floor(diffSec / 3600)} 小时前`;
    else if (diffSec < 86400 * 30) relative = `${Math.floor(diffSec / 86400)} 天前`;
    else if (diffSec < 86400 * 365) relative = `${Math.floor(diffSec / (86400 * 30))} 个月前`;
    else relative = `${Math.floor(diffSec / (86400 * 365))} 年前`;
  } else {
    const future = Math.abs(diffSec);
    if (future < 60) relative = `${future} 秒后`;
    else if (future < 3600) relative = `${Math.floor(future / 60)} 分钟后`;
    else if (future < 86400) relative = `${Math.floor(future / 3600)} 小时后`;
    else if (future < 86400 * 30) relative = `${Math.floor(future / 86400)} 天后`;
    else if (future < 86400 * 365) relative = `${Math.floor(future / (86400 * 30))} 个月后`;
    else relative = `${Math.floor(future / (86400 * 365))} 年后`;
  }

  const rows: [string, string][] = [
    ["原始输入", `\`${trimmed}\``],
    ["本地时间", localFormatted],
    ["UTC 时间", utcFormatted],
    ["ISO 8601", `\`${isoFormatted}\``],
    ["秒级时间戳", `\`${sec}\``],
    ["毫秒级时间戳", `\`${ms}\``],
    ["浮点时间戳", `\`${pythonFloat}\``],
    ["相对时间 (Relative)", relative],
  ];

  if (isSingleNumber) {
    rows.splice(
      1,
      0,
      ["英语习惯 (EN)", formatEnglishNumber(singleNumVal)],
      ["中文习惯 (CN)", formatChineseNumber(singleNumVal)],
    );
  }

  let md = "| 格式 / 属性 | 数值 / 结果 |\n| :--- | :--- |\n";
  for (const [prop, val] of rows) {
    md += `| **${prop}** | ${val} |\n`;
  }
  return md;
}

/**
 * 5. 智能计算器 (带中英文数量单位换算)
 */
export function evaluateSmartExpression(input: string): string {
  const trimmed = input.trim();
  if (!trimmed) {
    throw new Error("请输入要计算的数学表达式");
  }

  // Replace symbols
  let expr = trimmed.replace(/×/g, "*").replace(/÷/g, "/").replace(/\^/g, "**");

  // Replace units with exponential notation (e.g. 1K -> 1e3, 1万 -> 1e4)
  // Order matters: match larger / multi-character units first!
  const unitReplacements: [RegExp, string][] = [
    [/(\d+(?:\.\d+)?)\s*(?:万亿|兆|[zZtT])/g, "$1e12"],
    [/(\d+(?:\.\d+)?)\s*[pP]/g, "$1e15"],
    [/(\d+(?:\.\d+)?)\s*(?:千万)/g, "$1e7"],
    [/(\d+(?:\.\d+)?)\s*(?:百万|[mM])/g, "$1e6"],
    [/(\d+(?:\.\d+)?)\s*(?:[gG]|B)/g, "$1e9"],
    [/(\d+(?:\.\d+)?)\s*(?:亿|[yY])/g, "$1e8"],
    [/(\d+(?:\.\d+)?)\s*(?:万|[wW])/g, "$1e4"],
    [/(\d+(?:\.\d+)?)\s*(?:千|[qQkK])/g, "$1e3"],
    [/(\d+(?:\.\d+)?)\s*(?:百|b)/g, "$1e2"],
  ];

  for (const [pattern, replacement] of unitReplacements) {
    expr = expr.replace(pattern, replacement);
  }

  // Security & syntax check: expression must only contain allowed math characters
  if (!/^[0-9eE+\-*/%(). ]+$/.test(expr)) {
    throw new Error("无法识别的数学表达式 (包含非法字符或未识别单位)");
  }

  let result: number;
  try {
    const fn = new Function('"use strict"; return (' + expr + ")");
    result = Number(fn());
  } catch (err: any) {
    throw new Error("数学表达式语法错误: " + (err?.message || "无法计算"));
  }

  if (isNaN(result) || !isFinite(result)) {
    throw new Error("计算结果为无效数值 (NaN 或 无穷大)");
  }

  const rows: [string, string][] = [
    ["输入表达式 (Input)", `\`${trimmed}\``],
    ["精确值 (Exact)", formatExactNumber(result)],
    ["英语习惯 (EN)", formatEnglishNumber(result)],
    ["中文习惯 (CN)", formatChineseNumber(result)],
  ];

  let md = "| 格式 / 维度 | 结算结果 |\n| :--- | :--- |\n";
  for (const [prop, val] of rows) {
    md += `| **${prop}** | ${val} |\n`;
  }
  return md;
}

/**
 * 统一代码型工具调度器
 */
export function executeCodeTool(
  toolId: string,
  input: string,
  preprocessed?: PreprocessedResult,
): string {
  switch (toolId) {
    case "json-formatter":
      // If Rust already preprocessed pretty json, use it directly, else fallback to formatJson
      if (preprocessed?.formattedText) {
        return preprocessed.formattedText;
      }
      return formatJson(input);

    case "url-codec":
      return parseUrl(input);

    case "jwt-inspector":
      return inspectJwt(input);

    case "timestamp-converter":
      return convertTimestamp(input);

    case "calculator":
      return evaluateSmartExpression(input);

    default:
      if (preprocessed?.formattedText) {
        return preprocessed.formattedText;
      }
      return input;
  }
}
