import { FiatCurrency } from "store/features/settingsSlice";

/**
 * Returns the symbol for a given fiat currency.
 * @param currency The fiat currency to get the symbol for.
 * @returns The symbol for the given fiat currency, or null if the currency is not supported.
 */
export function currencySymbol(currency: FiatCurrency): string | null {
    switch (currency) {
        case FiatCurrency.Usd: return "$";
        case FiatCurrency.Eur: return "€";
        case FiatCurrency.Gbp: return "£";
        case FiatCurrency.Chf: return "₣";
        case FiatCurrency.Jpy: return "¥";
        case FiatCurrency.Ars: return "$";
        case FiatCurrency.Aud: return "$";
        case FiatCurrency.Cad: return "$";
        case FiatCurrency.Cny: return "¥";
        case FiatCurrency.Czk: return "Kč";
        case FiatCurrency.Dkk: return "kr";
        case FiatCurrency.Gel: return "₾";
        case FiatCurrency.Hkd: return "HK$";
        case FiatCurrency.Ils: return "₪";
        case FiatCurrency.Inr: return "₹";
        case FiatCurrency.Krw: return "₩";
        case FiatCurrency.Kwd: return "KD";
        case FiatCurrency.Lkr: return "₨";
        case FiatCurrency.Mmk: return "K";
        case FiatCurrency.Mxn: return "$";
        case FiatCurrency.Nok: return "kr";
        case FiatCurrency.Nzd: return "$";
        case FiatCurrency.Php: return "₱";
        case FiatCurrency.Pkr: return "₨";
        case FiatCurrency.Pln: return "zł";
        case FiatCurrency.Rub: return "₽";
        case FiatCurrency.Sar: return "﷼";
        case FiatCurrency.Sek: return "kr";
        case FiatCurrency.Sgd: return "$";
        case FiatCurrency.Thb: return "฿";
        case FiatCurrency.Try: return "₺";
        case FiatCurrency.Twd: return "NT$";
        case FiatCurrency.Uah: return "₴";
        case FiatCurrency.Vef: return "Bs";
        case FiatCurrency.Vnd: return "₫";
        case FiatCurrency.Zar: return "R";
        default: return null;
    }
}