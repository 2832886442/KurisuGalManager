import { useState, useEffect } from 'react';
import { useGame } from '../hooks/useGameData';
import Icon from './Icon';

export default function GlobalLoader() {
  const { state, dispatch } = useGame();
  const [showCancel, setShowCancel] = useState(false);

  useEffect(() => {
    if (!state.loading) {
      setShowCancel(false);
      return;
    }
    const timer = setTimeout(() => {
      setShowCancel(true);
    }, 8000);
    return () => clearTimeout(timer);
  }, [state.loading]);

  if (!state.loading) return null;

  const handleCancel = () => {
    window.dispatchEvent(new CustomEvent('loading-cancel'));
    dispatch({ type: 'CANCEL_LOADING' });
  };

  return (
    <div className="global-loader">
      <div className="loader-spinner"></div>
      <span className="loader-text">{state.loading}</span>
      {showCancel && (
        <div className="loader-cancel-area">
          <span className="loader-cancel-hint">长时间加载可能出现问题</span>
          <button className="loader-cancel-btn" onClick={handleCancel} title="取消加载">
            <Icon name="x" size={16} />
          </button>
        </div>
      )}
    </div>
  );
}
