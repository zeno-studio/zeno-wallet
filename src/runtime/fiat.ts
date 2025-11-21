
import type { FiatCode, FiatRates } from './types';

export function formatCurrencySymbol(code: FiatCode): string {
  switch (code) {
    case "USD": return "$";
    case "EUR": return "€";
    case "GBP": return "£";
    case "JPY": return "¥";
    case "CNY": return "¥";
    case "KRW": return "₩";

    case "SGD": return "S$";
    case "VND": return "₫";
    case "MYR": return "RM";
    case "IDR": return "Rp";
    case "THB": return "฿";
    case "PHP": return "₱";

    case "INR": return "₹";
    case "PKR": return "Rs";

    case "VES": return "Bs";
    case "ARS": return "$";
    case "BRL": return "R$";
    case "CLP": return "$";
    case "COP": return "$";
    case "PEN": return "S/";

    case "CHF": return "CHF";
    case "CAD": return "C$";
    case "AUD": return "A$";
    case "NZD": return "NZ$";

    default: return code.toUpperCase();
  }
}


// 输出带符号的金额，比如：
// USD → $123.45
export function formatCurrencyAmount(code: FiatCode, rate: FiatRates): string {
  const symbol = formatCurrencySymbol(code);
  return `${symbol}${rate[code].toLocaleString(undefined, {
    maximumFractionDigits: 2,
  })}`;
}




