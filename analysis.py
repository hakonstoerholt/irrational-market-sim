import pandas as pd
import mplfinance as mpf
import matplotlib.pyplot as plt

df = pd.read_csv('engine/trades.csv')

df['price'] = df['price'] / 100.0 
df.index = pd.to_datetime(df['timestamp'] * 0.1, unit='s', origin='unix')

print(f"Loaded {len(df)} trades.")
print(df.head())

ohlc = df['price'].resample('5s').ohlc()
volume = df['amount'].resample('5s').sum()

ohlc.dropna(inplace=True)
volume = volume.reindex(ohlc.index).fillna(0)

market_data = ohlc.copy()
market_data['volume'] = volume

mpf.plot(
    market_data,
    type='candle',
    style='charles',
    title='Market Simulator - Simulation Results',
    ylabel='Price ($)',
    volume=True,
    mav=(3, 6),
    savefig='market_analysis.png'
)

print("Chart saved to market_analysis.png 📈")

buyer_counts = df['buyer_id'].value_counts()
seller_counts = df['seller_id'].value_counts()

print("\n--- Top Buyers ---")
print(buyer_counts.head(5))

print("\n--- Top Sellers ---")
print(seller_counts.head(5))

print("\n--- Agent Strategy Performance ---")
agents_info = {}

for i in range(1, 11):
    agents_info[i] = 'RandomWalker'
for i in range(11, 16):
    agents_info[i] = 'TrendFollower'
for i in range(16, 21):
    agents_info[i] = 'MeanReverter'

df['buyer_strategy'] = df['buyer_id'].map(agents_info)
df['seller_strategy'] = df['seller_id'].map(agents_info)

print("Trades by Buyer Strategy:")
print(df['buyer_strategy'].value_counts())
print("\nTrades by Seller Strategy:")
print(df['seller_strategy'].value_counts())
