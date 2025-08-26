using System;
using System.Runtime.InteropServices;

namespace TradeChestGUI;

[StructLayout(LayoutKind.Sequential)]
public struct Quote
{
    public double Bid;
    public double Ask;
    public double Mid;
    public int Inventory;
    public double MarketBid;
    public double MarketAsk;
    public double Spread;
    public double UsdBalance;
    public double BtcBalance;
    public double Pnl;
    public ulong LatencyUs;
}

public unsafe class RustCore : IDisposable
{
    private IntPtr _core;

    [DllImport("./libtradechest_core.dylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr create_core(byte[] symbol);

    [DllImport("./libtradechest_core.dylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern void start_market_data(IntPtr core);

    [DllImport("./libtradechest_core.dylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern Quote get_current_quote(IntPtr core);

    [DllImport("./libtradechest_core.dylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern void destroy_core(IntPtr core);

    [DllImport("./libtradechest_core.dylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern void set_initial_portfolio(IntPtr core, double usd, double btc);

    [DllImport("./libtradechest_core.dylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern int simulate_buy_trade(IntPtr core, int quantity);

    [DllImport("./libtradechest_core.dylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern int simulate_sell_trade(IntPtr core, int quantity);

    [DllImport("./libtradechest_core.dylib", CallingConvention = CallingConvention.Cdecl)]
    private static extern int auto_trade(IntPtr core, byte[] result, int len);

    public RustCore(string symbol)
    {
        var symbolBytes = System.Text.Encoding.UTF8.GetBytes(symbol + "\0");
        _core = create_core(symbolBytes);
    }

    public void StartMarketData() => start_market_data(_core);
    public Quote GetQuote() => get_current_quote(_core);
    public void SetPortfolio(double usd, double btc) => set_initial_portfolio(_core, usd, btc);
    public bool SimulateBuy(int quantity) => simulate_buy_trade(_core, quantity) == 1;
    public bool SimulateSell(int quantity) => simulate_sell_trade(_core, quantity) == 1;
    
    public string AutoTrade()
    {
        var buffer = new byte[256];
        return auto_trade(_core, buffer, buffer.Length) == 1 
            ? System.Text.Encoding.UTF8.GetString(buffer).TrimEnd('\0') 
            : null;
    }

    public void Dispose()
    {
        if (_core != IntPtr.Zero)
        {
            destroy_core(_core);
            _core = IntPtr.Zero;
        }
    }
}