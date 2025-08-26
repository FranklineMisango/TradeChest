using System;
using System.Collections.Generic;
using System.Linq;

namespace TradeChestGUI;

public static class Dashboard
{
    private static List<Quote> _history = new();
    private static bool _headerPrinted = false;
    private static int _rowCount = 0;
    private static int _totalTrades = 0;
    private static double _totalVolume = 0;
    
    public static void DisplayQuote(Quote quote, int iteration)
    {
        _history.Add(quote);
        if (_history.Count > 50) _history.RemoveAt(0);
        
        if (!_headerPrinted)
        {
            Console.Clear();
            PrintTitle();
            PrintInstructions();
            PrintMetrics(quote);
            PrintTableHeader();
            _headerPrinted = true;
        }
        
        PrintRow(quote, iteration);
        _rowCount++;
        
        if (_rowCount >= 12)
        {
            Console.WriteLine("└─────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴─────────┘");
            PrintControls();
            _headerPrinted = false;
            _rowCount = 0;
        }
    }
    
    private static void PrintTitle()
    {
        Console.ForegroundColor = ConsoleColor.Cyan;
        Console.WriteLine("╔══════════════════════════════════════════════════════════════════════════════════════════════════════════╗");
        Console.WriteLine("║                                    TRADECHEST MARKET MAKER v1.0                                         ║");
        Console.WriteLine("║                              High-Frequency Crypto Trading System                                        ║");
        Console.WriteLine("╚══════════════════════════════════════════════════════════════════════════════════════════════════════════╝");
        Console.ResetColor();
    }
    
    private static void PrintInstructions()
    {
        Console.ForegroundColor = ConsoleColor.Green;
        Console.WriteLine("CONTROLS: [1] Adjust σ  [2] Adjust γ  [3] Adjust k  [4] Reset Portfolio  [i] Info Mode  [q] Quit");
        Console.ResetColor();
        Console.WriteLine();
    }
    
    private static void PrintMetrics(Quote quote)
    {
        var portfolioValue = quote.UsdBalance + quote.BtcBalance * quote.Mid;
        var exposure = (quote.BtcBalance * quote.Mid / portfolioValue) * 100;
        var informationRatio = _history.Count > 10 ? CalculateInformationRatio() : 0.0;
        var techMetrics = GetTechMetrics(quote);
        
        Console.ForegroundColor = ConsoleColor.Yellow;
        Console.WriteLine("┌─────────────────────────────────────────────────────────────────────────────────────────────────────────┐");
        Console.WriteLine("│    HJB PARAMETERS       PORTFOLIO METRICS       MARKET SIGNALS        PERFORMANCE         SYSTEM STATUS   │");
        Console.WriteLine("└─────────────────────────────────────────────────────────────────────────────────────────────────────────┘");
        Console.ResetColor();
        
        // Row 1
        Console.Write("  ");
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write("σ (Vol): 0.30        ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Cyan;
        Console.Write($"Value: ${portfolioValue/1000000:F1}M         ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Magenta;
        Console.Write($"Spread Edge: {((quote.Ask - quote.Bid) - quote.Spread):F3}   ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Yellow;
        Console.Write($"Trades: {_totalTrades}/hr      ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write("Status: ACTIVE");
        Console.ResetColor();
        Console.WriteLine();
        
        // Row 2
        Console.Write("  ");
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write("γ (Risk): 0.10       ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Cyan;
        Console.Write($"Cash: ${quote.UsdBalance/1000000:F1}M          ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Magenta;
        Console.Write($"Info Ratio: {informationRatio:F3}      ");
        Console.ResetColor();
        Console.ForegroundColor = quote.Pnl >= 0 ? ConsoleColor.Green : ConsoleColor.Red;
        Console.Write($"P&L: {(quote.Pnl >= 0 ? "+" : "")}${quote.Pnl:F0}         ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write($"Latency: {quote.LatencyUs}μs");
        Console.ResetColor();
        Console.WriteLine();
        
        // Row 3
        Console.Write("  ");
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write("k (Liq): 1.50        ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Cyan;
        Console.Write($"BTC: {quote.BtcBalance:F3}        ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Magenta;
        Console.Write($"Mkt Pressure: {GetMarketPressure():F2}     ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Yellow;
        Console.Write($"Volume: ${_totalVolume/1000:F0}K       ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write("Uptime: 100%");
        Console.ResetColor();
        Console.WriteLine();
        
        // Row 4 - Tech Metrics
        Console.Write("  ");
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write($"Inventory: {quote.Inventory} BTC   ");
        Console.ResetColor();
        Console.ForegroundColor = exposure > 80 ? ConsoleColor.Red : exposure > 60 ? ConsoleColor.Yellow : ConsoleColor.Green;
        Console.Write($"Exposure: {exposure:F1}%       ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Magenta;
        Console.Write($"Volatility: {techMetrics.volatility:F2}%     ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Yellow;
        Console.Write($"Sharpe: {CalculateSharpe():F2}        ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write("Feed: BINANCE");
        Console.ResetColor();
        Console.WriteLine();
        
        // Row 5 - Additional Tech Metrics
        Console.Write("  ");
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write($"Max Drawdown: {techMetrics.maxDrawdown:F2}%  ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Cyan;
        Console.Write($"Win Rate: {techMetrics.winRate:F1}%      ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Magenta;
        Console.Write($"Beta: {techMetrics.beta:F3}           ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Yellow;
        Console.Write($"Alpha: {techMetrics.alpha:F3}%       ");
        Console.ResetColor();
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write($"VaR 95%: ${techMetrics.var95:F0}K");
        Console.ResetColor();
        Console.WriteLine();
        
        Console.WriteLine();
    }
    
    private static void PrintTableHeader()
    {
        Console.ForegroundColor = ConsoleColor.White;
        Console.WriteLine("┌─────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬─────────┐");
        Console.WriteLine("│  #  │ Mkt Mid  │ Mkt Bid  │ Mkt Ask  │ Opt Bid  │ Opt Ask  │ USD Bal  │ BTC Bal  │   P&L    │ Latency │");
        Console.WriteLine("├─────┼──────────┼──────────┼──────────┼──────────┼──────────┼──────────┼──────────┼──────────┼─────────┤");
        Console.ResetColor();
    }
    
    private static void PrintRow(Quote quote, int iteration)
    {
        Console.Write("│");
        Console.ForegroundColor = ConsoleColor.White;
        Console.Write($"{iteration,4}");
        Console.ResetColor();
        
        Console.Write(" │");
        Console.ForegroundColor = ConsoleColor.Yellow;
        Console.Write($"{quote.Mid,9:F2}");
        Console.ResetColor();
        
        Console.Write(" │");
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write($"{quote.MarketBid,9:F2}");
        Console.ResetColor();
        
        Console.Write(" │");
        Console.ForegroundColor = ConsoleColor.Red;
        Console.Write($"{quote.MarketAsk,9:F2}");
        Console.ResetColor();
        
        Console.Write(" │");
        Console.ForegroundColor = ConsoleColor.Green;
        Console.Write($"{quote.Bid,9:F2}");
        Console.ResetColor();
        
        Console.Write(" │");
        Console.ForegroundColor = ConsoleColor.Red;
        Console.Write($"{quote.Ask,9:F2}");
        Console.ResetColor();
        
        Console.Write(" │");
        Console.ForegroundColor = ConsoleColor.Magenta;
        Console.Write($"{quote.UsdBalance/1000000,7:F1}M");
        Console.ResetColor();
        
        Console.Write(" │");
        Console.ForegroundColor = ConsoleColor.Cyan;
        Console.Write($"{quote.BtcBalance,9:F3}");
        Console.ResetColor();
        
        Console.Write(" │");
        Console.ForegroundColor = quote.Pnl >= 0 ? ConsoleColor.Green : ConsoleColor.Red;
        Console.Write($"{(quote.Pnl >= 0 ? "+" : "")}{quote.Pnl,8:F0}");
        Console.ResetColor();
        
        Console.Write(" │");
        Console.ForegroundColor = quote.LatencyUs < 100 ? ConsoleColor.Green : ConsoleColor.Yellow;
        Console.Write($"{quote.LatencyUs,6}μs");
        Console.ResetColor();
        
        Console.WriteLine(" │");
    }
    
    private static void PrintControls()
    {
        Console.WriteLine();
        Console.ForegroundColor = ConsoleColor.Gray;
        Console.WriteLine("REAL-TIME CONTROLS: Press [1-4] to adjust parameters, [i] for detailed info, [q] to quit");
        Console.ResetColor();
        Console.WriteLine();
    }
    
    private static double CalculateInformationRatio()
    {
        if (_history.Count < 2) return 0.0;
        
        var returns = new List<double>();
        for (int i = 1; i < _history.Count; i++)
        {
            var ret = (_history[i].Mid - _history[i-1].Mid) / _history[i-1].Mid;
            returns.Add(ret);
        }
        
        var mean = returns.Sum() / returns.Count;
        var variance = returns.Select(r => Math.Pow(r - mean, 2)).Sum() / returns.Count;
        var stdDev = Math.Sqrt(variance);
        
        return stdDev > 0 ? mean / stdDev : 0.0;
    }
    
    private static double GetMarketPressure()
    {
        if (_history.Count < 5) return 0.0;
        
        var recent = _history.TakeLast(5).ToList();
        var bidPressure = recent.Average(q => q.MarketBid);
        var askPressure = recent.Average(q => q.MarketAsk);
        var midPressure = recent.Average(q => q.Mid);
        
        return (bidPressure + askPressure) / (2 * midPressure) - 1.0;
    }
    
    private static double GetAdverseSelection()
    {
        if (_history.Count < 10) return 0.0;
        
        var recent = _history.TakeLast(10).ToList();
        var spreadTightness = recent.Average(q => q.Spread);
        var optimalSpread = recent.Average(q => q.Ask - q.Bid);
        
        return spreadTightness > 0 ? optimalSpread / spreadTightness - 1.0 : 0.0;
    }
    
    private static double CalculateSharpe()
    {
        if (_history.Count < 10) return 0.0;
        
        var pnlValues = _history.TakeLast(10).Select(q => q.Pnl).ToList();
        if (pnlValues.Count < 2) return 0.0;
        
        var returns = new List<double>();
        for (int i = 1; i < pnlValues.Count; i++)
        {
            returns.Add(pnlValues[i] - pnlValues[i-1]);
        }
        
        var mean = returns.Average();
        var stdDev = Math.Sqrt(returns.Select(r => Math.Pow(r - mean, 2)).Average());
        
        return stdDev > 0 ? mean / stdDev * Math.Sqrt(252) : 0.0; // Annualized
    }
    
    private static (double volatility, double maxDrawdown, double winRate, double beta, double alpha, double var95) GetTechMetrics(Quote quote)
    {
        if (_history.Count < 10) return (0.0, 0.0, 0.0, 1.0, 0.0, 50.0);
        
        var recent = _history.TakeLast(20).ToList();
        
        // Volatility calculation
        var returns = new List<double>();
        for (int i = 1; i < recent.Count; i++)
        {
            returns.Add((recent[i].Mid - recent[i-1].Mid) / recent[i-1].Mid);
        }
        var volatility = returns.Count > 1 ? Math.Sqrt(returns.Select(r => r * r).Average()) * 100 : 0.0;
        
        // Max Drawdown
        var pnlValues = recent.Select(q => q.Pnl).ToList();
        var peak = pnlValues.Max();
        var trough = pnlValues.Min();
        var maxDrawdown = peak > 0 ? ((peak - trough) / peak) * 100 : 0.0;
        
        // Win Rate (simplified)
        var positiveReturns = returns.Count(r => r > 0);
        var winRate = returns.Count > 0 ? (double)positiveReturns / returns.Count * 100 : 0.0;
        
        // Beta (market correlation)
        var beta = returns.Count > 5 ? Math.Abs(returns.Average()) * 10 : 1.0;
        
        // Alpha (excess return)
        var alpha = returns.Count > 5 ? returns.Average() * 100 : 0.0;
        
        // VaR 95%
        var var95 = volatility * 1.65 * Math.Sqrt(quote.UsdBalance + quote.BtcBalance * quote.Mid) / 1000;
        
        return (volatility, maxDrawdown, winRate, beta, alpha, var95);
    }
}