using System;
using System.Threading;
using TradeChestGUI;

var core = new RustCore("BTCUSDT");

Console.WriteLine("TradeChest Market Maker Started - Connecting to Binance...");
core.SetPortfolio(10_000_000.0, 100.0); // $10M USD + 100 BTC
core.StartMarketData();

Console.WriteLine("Waiting for market data...");
Thread.Sleep(5000); // Wait for WebSocket connection

int updateCount = 0;
string lastTradeMsg = "";

while (true)
{
    updateCount++;
    var quote = core.GetQuote();
    
    if (quote.Mid > 0)
    {
        // Execute automatic trading
        var tradeResult = core.AutoTrade();
        if (tradeResult != null)
        {
            lastTradeMsg = tradeResult;
        }
        
        Dashboard.DisplayQuote(quote, updateCount);
        
        if (!string.IsNullOrEmpty(lastTradeMsg))
        {
            Console.WriteLine($"TRADE: {lastTradeMsg}");
            if (updateCount % 5 == 0) lastTradeMsg = ""; // Clear after showing for a while
        }
        
        // Check for user input
        if (Console.IsInputRedirected == false && Console.KeyAvailable)
        {
            var key = Console.ReadKey(true).KeyChar;
            switch (key)
            {
                case '1':
                    Console.WriteLine("\nAdjust σ (Volatility): Enter new value (0.1-1.0):");
                    break;
                case '2':
                    Console.WriteLine("\nAdjust γ (Risk Aversion): Enter new value (0.01-0.5):");
                    break;
                case '3':
                    Console.WriteLine("\nAdjust k (Liquidity): Enter new value (0.5-3.0):");
                    break;
                case '4':
                    Console.WriteLine("\nResetting portfolio to initial state...");
                    core.SetPortfolio(10_000_000.0, 100.0);
                    break;
                case 'i':
                    Console.WriteLine("\n=== INFORMATION MODE ===");
                    Console.WriteLine("Market Microstructure Analysis:");
                    Console.WriteLine("- Adverse Selection: Measures information disadvantage");
                    Console.WriteLine("- Market Pressure: Indicates directional bias");
                    Console.WriteLine("- Spread Edge: Our advantage over market spread");
                    Console.WriteLine("Press any key to continue...");
                    Console.ReadKey();
                    break;
                case 'q':
                    core.Dispose();
                    return;
            }
        }
    }
    else
    {
        Console.WriteLine($"Connecting... ({updateCount})");
    }
    
    Thread.Sleep(1000);
}