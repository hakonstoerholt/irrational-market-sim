import React, { useState } from 'react';
import { Zap, Pause, Play, TrendingDown, TrendingUp, Rocket, AlertTriangle, Skull, Fish, DollarSign } from 'lucide-react';

const API_BASE = 'http://127.0.0.1:3000/api/admin';

function GodPanel() {
  const [isPaused, setIsPaused] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [status, setStatus] = useState('');
  
  // Magnitude states
  const [earningsSurprise, setEarningsSurprise] = useState(15);
  const [tariffSeverity, setTariffSeverity] = useState(3);
  const [rugPullMagnitude, setRugPullMagnitude] = useState(2);
  const [whaleMagnitude, setWhaleMagnitude] = useState(2);

  const showStatus = (message, duration = 3000) => {
    setStatus(message);
    setTimeout(() => setStatus(''), duration);
  };

  const handleFlashCrash = async () => {
    try {
      const res = await fetch(`${API_BASE}/crash`, { method: 'POST' });
      const data = await res.json();
      showStatus(data.message);
    } catch (err) {
      showStatus(`Error: ${err.message}`);
    }
  };

  const handleTogglePause = async () => {
    const action = isPaused ? 'resume' : 'pause';
    try {
      const res = await fetch(`${API_BASE}/control`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action }),
      });
      const data = await res.json();
      setIsPaused(!isPaused);
      showStatus(data.message);
    } catch (err) {
      showStatus(`Error: ${err.message}`);
    }
  };

  const handleBuyWall = async () => {
    try {
      const res = await fetch(`${API_BASE}/pump`, {
        method: 'POST',
      });
      const data = await res.json();
      showStatus(data.message);
    } catch (err) {
      showStatus(`Error: ${err.message}`);
    }
  };

  const handleSellWall = async () => {
    try {
      const res = await fetch(`${API_BASE}/dump`, {
        method: 'POST',
      });
      const data = await res.json();
      showStatus(data.message);
    } catch (err) {
      showStatus(`Error: ${err.message}`);
    }
  };

  const handleEarnings = async () => {
    try {
      const surprise_pct = Number(earningsSurprise);
      const res = await fetch(`${API_BASE}/earnings`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ surprise_pct }),
      });
      const data = await res.json();
      showStatus(data.message);
    } catch (err) {
      showStatus(`Error: ${err.message}`);
    }
  };

  const handleTariffs = async () => {
    try {
      const res = await fetch(`${API_BASE}/tariffs`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ severity: tariffSeverity }),
      });
      const data = await res.json();
      showStatus(data.message);
    } catch (err) {
      showStatus(`Error: ${err.message}`);
    }
  };

  const handleRugPull = async () => {
    try {
      const res = await fetch(`${API_BASE}/rugpull`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ magnitude: rugPullMagnitude }),
      });
      const data = await res.json();
      showStatus(data.message);
    } catch (err) {
      showStatus(`Error: ${err.message}`);
    }
  };

  const handleWhale = async () => {
    try {
      const res = await fetch(`${API_BASE}/whale`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ magnitude: whaleMagnitude }),
      });
      const data = await res.json();
      showStatus(data.message);
    } catch (err) {
      showStatus(`Error: ${err.message}`);
    }
  };

  return (
    <>
      {/* Floating Toggle Button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="fixed bottom-8 right-8 w-16 h-16 bg-purple-600 hover:bg-purple-700 rounded-full shadow-lg flex items-center justify-center transition-all z-50"
      >
        <Zap className="w-8 h-8 text-white" />
      </button>

      {/* God Panel Sidebar */}
      {isOpen && (
        <div className="fixed right-0 top-0 h-full w-80 bg-gray-900 border-l-2 border-purple-600 shadow-2xl z-40 p-6 overflow-y-auto">
          <div className="mb-6">
            <h2 className="text-2xl font-bold text-purple-400 flex items-center gap-2">
              <Zap className="w-6 h-6" />
              God Mode
            </h2>
            <p className="text-gray-500 text-sm mt-1">Manual Market Control</p>
          </div>

          {/* Status Message */}
          {status && (
            <div className="mb-4 p-3 bg-purple-900/50 border border-purple-600 rounded-lg text-purple-300 text-sm">
              {status}
            </div>
          )}

          {/* Control Buttons */}
          <div className="space-y-3">
            {/* Pause/Resume */}
            <button
              onClick={handleTogglePause}
              className={`w-full py-3 px-4 rounded-lg font-semibold flex items-center justify-center gap-2 transition-all ${
                isPaused
                  ? 'bg-green-600 hover:bg-green-700 text-white'
                  : 'bg-yellow-600 hover:bg-yellow-700 text-white'
              }`}
            >
              {isPaused ? (
                <>
                  <Play className="w-5 h-5" />
                  Resume Simulation
                </>
              ) : (
                <>
                  <Pause className="w-5 h-5" />
                  Pause Simulation
                </>
              )}
            </button>

            {/* Flash Crash */}
            <button
              onClick={handleFlashCrash}
              className="w-full py-3 px-4 bg-red-600 hover:bg-red-700 text-white rounded-lg font-semibold flex items-center justify-center gap-2 transition-all"
            >
              <TrendingDown className="w-5 h-5" />
              Flash Crash
            </button>

            {/* Pump */}
            <button
              onClick={handleBuyWall}
              className="w-full py-3 px-4 bg-green-600 hover:bg-green-700 text-white rounded-lg font-semibold flex items-center justify-center gap-2 transition-all"
            >
              <Rocket className="w-5 h-5" />
              Pump Market
            </button>

            {/* Dump */}
            <button
              onClick={handleSellWall}
              className="w-full py-3 px-4 bg-orange-600 hover:bg-orange-700 text-white rounded-lg font-semibold flex items-center justify-center gap-2 transition-all"
            >
              <TrendingDown className="w-5 h-5" />
              Dump Market
            </button>

            {/* Earnings */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-400">Earnings Surprise: {earningsSurprise > 0 ? '+' : ''}{earningsSurprise}%</label>
                <input
                  type="range"
                  min="-30"
                  max="30"
                  value={earningsSurprise}
                  onChange={(e) => setEarningsSurprise(Number(e.target.value))}
                  className="w-32"
                />
              </div>
              <button
                onClick={handleEarnings}
                className={`w-full py-2 px-3 rounded-lg text-white text-sm font-semibold flex items-center justify-center gap-2 transition-all ${earningsSurprise >= 0 ? 'bg-emerald-600 hover:bg-emerald-700' : 'bg-rose-600 hover:bg-rose-700'}`}
              >
                {earningsSurprise >= 0 ? (
                  <DollarSign className="w-4 h-4" />
                ) : (
                  <TrendingDown className="w-4 h-4" />
                )}
                Announce Earnings
              </button>
            </div>

            {/* Tariffs */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-400">Tariff Severity: {tariffSeverity.toFixed(1)}</label>
                <input
                  type="range"
                  min="1"
                  max="10"
                  step="0.5"
                  value={tariffSeverity}
                  onChange={(e) => setTariffSeverity(Number(e.target.value))}
                  className="w-32"
                />
              </div>
              <button
                onClick={handleTariffs}
                className="w-full py-2 px-3 bg-amber-600 hover:bg-amber-700 text-white rounded-lg text-sm font-semibold flex items-center justify-center gap-2 transition-all"
              >
                <AlertTriangle className="w-4 h-4" />
                Announce Tariffs
              </button>
            </div>

            {/* Rug Pull */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-400">Rug Pull: {rugPullMagnitude.toFixed(1)}x (-{(rugPullMagnitude * 15).toFixed(0)}%)</label>
                <input
                  type="range"
                  min="0.5"
                  max="5"
                  step="0.5"
                  value={rugPullMagnitude}
                  onChange={(e) => setRugPullMagnitude(Number(e.target.value))}
                  className="w-32"
                />
              </div>
              <button
                onClick={handleRugPull}
                className="w-full py-2 px-3 bg-red-700 hover:bg-red-800 text-white rounded-lg text-sm font-semibold flex items-center justify-center gap-2 transition-all"
              >
                <Skull className="w-4 h-4" />
                Rug Pull
              </button>
            </div>

            {/* Whale Accumulation */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-400">Whale Size: {whaleMagnitude.toFixed(1)}x</label>
                <input
                  type="range"
                  min="0.5"
                  max="5"
                  step="0.5"
                  value={whaleMagnitude}
                  onChange={(e) => setWhaleMagnitude(Number(e.target.value))}
                  className="w-32"
                />
              </div>
              <button
                onClick={handleWhale}
                className="w-full py-2 px-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-semibold flex items-center justify-center gap-2 transition-all"
              >
                <Fish className="w-4 h-4" />
                Whale Accumulation
              </button>
            </div>
          </div>

          {/* Info Section */}
          <div className="mt-8 pt-6 border-t border-gray-700">
            <h3 className="text-sm font-semibold text-gray-400 mb-3">Event Types</h3>
            <ul className="text-xs text-gray-500 space-y-2">
              <li>• <span className="text-purple-400">Flash Crash</span>: Instant 60% dump</li>
              <li>• <span className="text-green-400">Pump/Dump</span>: Sustained pressure</li>
              <li>• <span className="text-emerald-400">Earnings</span>: Beat/miss with custom %</li>
              <li>• <span className="text-amber-400">Tariffs</span>: Macro sell pressure</li>
              <li>• <span className="text-red-400">Rug Pull</span>: Insider panic dump</li>
              <li>• <span className="text-blue-400">Whale</span>: Large sustained buying</li>
              <li>• <span className="text-purple-400">Pump</span>: 5x2K bids @ escalating prices</li>
              <li>• <span className="text-purple-400">Dump</span>: 5x2K asks @ cascading prices</li>
              <li>• <span className="text-purple-400">Pause</span>: Freezes all trading</li>
            </ul>
          </div>
        </div>
      )}
    </>
  );
}

export default GodPanel;
