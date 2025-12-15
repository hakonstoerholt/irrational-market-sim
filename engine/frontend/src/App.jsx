import React, { useEffect, useRef, useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { Activity, TrendingUp, TrendingDown } from 'lucide-react';

const WS_URL = 'ws://127.0.0.1:3000/ws';

function App() {
  // 1. The Data State
  // We store a rolling window of the last 100 ticks
  const [dataPoints, setDataPoints] = useState([]);
  const [currentPrice, setCurrentPrice] = useState(0);
  const [prevPrice, setPrevPrice] = useState(0);

  const wsRef = useRef(null);
  const reconnectTimerRef = useRef(null);
  const [connectionStatus, setConnectionStatus] = useState('Uninstantiated');
  
  // 2. Native WebSocket (avoids library compatibility issues)
  useEffect(() => {
    let isMounted = true;

    const connect = () => {
      if (!isMounted) return;
      setConnectionStatus('Connecting');

      try {
        const ws = new WebSocket(WS_URL);
        wsRef.current = ws;

        ws.onopen = () => {
          if (!isMounted) return;
          setConnectionStatus('Open');
        };

        ws.onclose = () => {
          if (!isMounted) return;
          setConnectionStatus('Closed');
          // Auto-reconnect after a short delay
          reconnectTimerRef.current = window.setTimeout(connect, 500);
        };

        ws.onerror = () => {
          if (!isMounted) return;
          setConnectionStatus('Error');
        };

        ws.onmessage = (event) => {
          let message;
          try {
            message = JSON.parse(event.data);
          } catch {
            return;
          }

          if (message?.type === 'ticker') {
            const { price, tick } = message;

            setCurrentPrice((prevCurrent) => {
              setPrevPrice(prevCurrent);
              return price;
            });

            setDataPoints((prev) => {
              const newData = [...prev, { tick, price: price / 100 }];
              if (newData.length > 50) return newData.slice(newData.length - 50);
              return newData;
            });
          }
        };
      } catch {
        setConnectionStatus('Error');
      }
    };

    connect();

    return () => {
      isMounted = false;
      if (reconnectTimerRef.current) window.clearTimeout(reconnectTimerRef.current);
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.close();
      }
      wsRef.current = null;
    };
  }, []);

  // Helper for color (Green if price went up, Red if down)
  const priceColor = currentPrice >= prevPrice ? 'text-green-500' : 'text-red-500';

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8 font-mono">
      {/* Header */}
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <Activity className="text-blue-500" /> Market Sim v1
          </h1>
          <p className="text-gray-400 text-sm mt-1">Status: <span className={connectionStatus === 'Open' ? "text-green-400" : "text-red-400"}>{connectionStatus}</span></p>
        </div>
        
        {/* Big Price Display */}
        <div className="text-right">
          <p className="text-gray-400 text-sm">Current Price</p>
          <div className={`text-5xl font-bold flex items-center justify-end ${priceColor}`}>
             ${(currentPrice / 100).toFixed(2)}
             {currentPrice >= prevPrice ? <TrendingUp className="ml-2 w-8 h-8"/> : <TrendingDown className="ml-2 w-8 h-8"/>}
          </div>
        </div>
      </div>

      {/* Main Chart Area */}
      <div className="bg-gray-800 p-4 rounded-xl shadow-lg border border-gray-700 h-96">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={dataPoints}>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
            <XAxis dataKey="tick" stroke="#9CA3AF" />
            <YAxis domain={['auto', 'auto']} stroke="#9CA3AF" />
            <Tooltip 
              contentStyle={{ backgroundColor: '#1F2937', border: 'none' }}
              itemStyle={{ color: '#60A5FA' }}
            />
            <Line 
              type="monotone" 
              dataKey="price" 
              stroke="#3B82F6" 
              strokeWidth={2}
              dot={false}
              isAnimationActive={false} // Disable animation for high-freq updates
            />
          </LineChart>
        </ResponsiveContainer>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-3 gap-4 mt-8">
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <h3 className="text-gray-400 text-sm">Total Ticks</h3>
          <p className="text-2xl font-bold">{dataPoints.length > 0 ? dataPoints[dataPoints.length-1].tick : 0}</p>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
            {/* Placeholder for future data like Volume */}
          <h3 className="text-gray-400 text-sm">Volume (24h)</h3>
          <p className="text-2xl font-bold">---</p>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <h3 className="text-gray-400 text-sm">Volatility</h3>
          <p className="text-2xl font-bold text-yellow-500">Medium</p>
        </div>
      </div>
    </div>
  );
}

export default App;
