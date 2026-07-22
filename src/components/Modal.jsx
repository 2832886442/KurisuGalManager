import { useEffect, useRef } from 'react';
import Icon from './Icon';

/**
 * 通用弹窗容器(Playnite 风格)
 *
 * 关闭行为:只有 mousedown 与 mouseup 都发生在遮罩(overlay)上时才视为
 * "点击遮罩关闭"。这样在子窗口内按下鼠标、拖到遮罩上松开时不会误关。
 * 其他关闭方式:点击关闭按钮、按 ESC。
 *
 * @param {string} title - 标题(可选,不传则不渲染标题栏)
 * @param {string} icon - 标题图标名(可选)
 * @param {string} size - 尺寸:默认 600px,'sm'=420px,'lg'=760px
 * @param {function} onClose - 关闭回调
 * @param {boolean} closeOnOverlay - 点击遮罩关闭,默认 true
 * @param {React.ReactNode} children - 弹窗内容
 * @param {string} className - 附加到 .modal 的额外类名
 */
export default function Modal({
  title,
  icon,
  size,
  onClose,
  closeOnOverlay = true,
  confirmBeforeClose = false,
  children,
  className = '',
  overlayClassName = '',
}) {
  const mouseDownOnOverlay = useRef(false);

  useEffect(() => {
    const handleEsc = (e) => {
      if (e.key === 'Escape') handleClose();
    };
    window.addEventListener('keydown', handleEsc);
    return () => window.removeEventListener('keydown', handleEsc);
  }, [onClose, confirmBeforeClose]);

  const handleClose = () => {
    if (confirmBeforeClose && !window.confirm('有未保存的修改，确定放弃吗？')) return;
    if (onClose) onClose();
  };

  const sizeClass = size === 'sm' ? ' modal-sm' : size === 'lg' ? ' modal-lg' : '';

  const handleMouseDown = (e) => {
    mouseDownOnOverlay.current = (e.target === e.currentTarget);
  };

  const handleOverlayClick = (e) => {
    if (closeOnOverlay && mouseDownOnOverlay.current && e.target === e.currentTarget) {
      handleClose();
    }
    mouseDownOnOverlay.current = false;
  };

  return (
    <div
      className={`modal-overlay${overlayClassName ? ' ' + overlayClassName : ''}`}
      onMouseDown={handleMouseDown}
      onClick={handleOverlayClick}
    >
      <div className={`modal${sizeClass}${className ? ' ' + className : ''}`}>
        {onClose && (
          <button className="modal-close" onClick={handleClose} title="关闭" aria-label="关闭">
            <Icon name="x" size={16} />
          </button>
        )}
        {title && (
          <h3 className="modal-title">
            {icon && <Icon name={icon} size={18} />}
            <span>{title}</span>
          </h3>
        )}
        {children}
      </div>
    </div>
  );
}
