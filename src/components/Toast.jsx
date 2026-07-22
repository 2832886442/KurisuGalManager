import { useState, useCallback, useRef, useEffect } from 'react';
import Icon from './Icon';

let toastRef = null;

export function showToast(type, message, duration = 3500) {
  if (toastRef) toastRef(type, message, duration);
}

const TOAST_ICONS = {
  success: 'check-circle',
  error:   'x-circle',
  warning: 'alert-triangle',
  info:    'info',
};

export default function ToastContainer() {
  const [toasts, setToasts] = useState([]);
  const counterRef = useRef(0);

  const addToast = useCallback((type, message, duration = 3500) => {
    const id = ++counterRef.current;
    setToasts(prev => [...prev, { id, type, message, visible: false }]);
    requestAnimationFrame(() => {
      setToasts(prev => prev.map(t => t.id === id ? { ...t, visible: true } : t));
    });
    setTimeout(() => {
      setToasts(prev => prev.map(t => t.id === id ? { ...t, visible: false } : t));
      setTimeout(() => {
        setToasts(prev => prev.filter(t => t.id !== id));
      }, 300);
    }, duration);
  }, []);

  useEffect(() => {
    toastRef = addToast;
    return () => { toastRef = null; };
  }, [addToast]);

  return (
    <div className="toast-container">
      {toasts.map(t => (
        <div key={t.id} className={`toast toast-${t.type}${t.visible ? ' show' : ''}`}>
          <span className="toast-icon">
            <Icon name={TOAST_ICONS[t.type] || 'info'} size={16} />
          </span>
          <span className="toast-msg">{t.message}</span>
        </div>
      ))}
    </div>
  );
}
