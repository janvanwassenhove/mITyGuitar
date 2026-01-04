import { X } from "lucide-react";

interface AboutDialogProps {
  onClose: () => void;
}

export default function AboutDialog({ onClose }: AboutDialogProps) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()} style={{ maxWidth: '500px' }}>
        <div className="modal-header">
          <h2 style={{ margin: 0, fontSize: '24px', fontWeight: 700 }}>About mITyGuitar</h2>
          <button onClick={onClose} className="modal-close-btn" aria-label="Close">
            <X size={20} />
          </button>
        </div>
        
        <div className="modal-body" style={{ padding: '24px' }}>
          <div style={{ textAlign: 'center', marginBottom: '24px' }}>
            <div style={{ 
              fontSize: '48px', 
              marginBottom: '12px',
              filter: 'drop-shadow(0 4px 8px rgba(0, 0, 0, 0.3))'
            }}>
              🎸
            </div>
            <h3 style={{ margin: '0 0 8px 0', fontSize: '20px', fontWeight: 600 }}>
              mITyGuitar
            </h3>
            <p style={{ margin: 0, fontSize: '14px', color: 'rgba(255, 255, 255, 0.7)' }}>
              Version 1.0
            </p>
          </div>

          <div style={{ 
            background: 'rgba(255, 255, 255, 0.05)', 
            borderRadius: '8px', 
            padding: '16px',
            marginBottom: '20px',
            border: '1px solid rgba(255, 255, 255, 0.1)'
          }}>
            <p style={{ margin: '0 0 12px 0', lineHeight: '1.6' }}>
              Transform your Guitar Hero controller into a real musical instrument!
            </p>
            <p style={{ margin: 0, lineHeight: '1.6', color: 'rgba(255, 255, 255, 0.8)' }}>
              Play live, load songs, and create music with intuitive chord mappings 
              and professional-quality sound fonts.
            </p>
          </div>

          <div style={{ 
            background: 'rgba(59, 130, 246, 0.1)', 
            borderRadius: '8px', 
            padding: '16px',
            marginBottom: '20px',
            border: '1px solid rgba(59, 130, 246, 0.3)'
          }}>
            <p style={{ margin: '0 0 8px 0', fontSize: '13px', color: 'rgba(255, 255, 255, 0.7)' }}>
              For more information and updates:
            </p>
            <a 
              href="https://www.mITyJohn.com" 
              target="_blank" 
              rel="noopener noreferrer"
              style={{ 
                color: '#60a5fa', 
                fontWeight: 600,
                fontSize: '16px',
                textDecoration: 'none'
              }}
              onMouseEnter={(e) => e.currentTarget.style.textDecoration = 'underline'}
              onMouseLeave={(e) => e.currentTarget.style.textDecoration = 'none'}
            >
              www.mITyJohn.com
            </a>
          </div>

          <div style={{ 
            fontSize: '13px', 
            color: 'rgba(255, 255, 255, 0.6)',
            lineHeight: '1.6',
            padding: '12px',
            background: 'rgba(0, 0, 0, 0.2)',
            borderRadius: '6px'
          }}>
            <strong>💡 Need Help?</strong>
            <br />
            Controller diagnostics and debugging tools are available in the 
            <strong style={{ color: 'rgba(255, 255, 255, 0.9)' }}> Diagnostics </strong> 
            view (Settings → Diagnostics).
          </div>
        </div>

        <div className="modal-footer" style={{ padding: '16px 24px', borderTop: '1px solid rgba(255, 255, 255, 0.1)' }}>
          <button 
            onClick={onClose}
            style={{
              width: '100%',
              padding: '10px 20px',
              borderRadius: '6px',
              border: 'none',
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              color: 'white',
              fontWeight: 600,
              cursor: 'pointer',
              fontSize: '14px'
            }}
            onMouseEnter={(e) => e.currentTarget.style.opacity = '0.9'}
            onMouseLeave={(e) => e.currentTarget.style.opacity = '1'}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
