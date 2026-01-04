import { X, CheckCircle, XCircle } from "lucide-react";

interface SongUploadDialogProps {
  onClose: () => void;
  songName: string;
  isError?: boolean;
  errorMessage?: string;
}

export default function SongUploadDialog({ 
  onClose, 
  songName,
  isError = false,
  errorMessage 
}: SongUploadDialogProps) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()} style={{ maxWidth: '450px' }}>
        <div className="modal-header">
          <h2 style={{ margin: 0, fontSize: '22px', fontWeight: 700 }}>
            {isError ? 'Upload Failed' : 'Song Uploaded'}
          </h2>
          <button onClick={onClose} className="modal-close-btn" aria-label="Close">
            <X size={20} />
          </button>
        </div>
        
        <div className="modal-body" style={{ padding: '24px' }}>
          <div style={{ textAlign: 'center', marginBottom: '20px' }}>
            <div style={{ 
              display: 'inline-flex',
              padding: '16px',
              borderRadius: '50%',
              background: isError ? 'rgba(239, 68, 68, 0.15)' : 'rgba(34, 197, 94, 0.15)',
              marginBottom: '12px'
            }}>
              {isError ? (
                <XCircle size={48} style={{ color: '#ef4444' }} />
              ) : (
                <CheckCircle size={48} style={{ color: '#22c55e' }} />
              )}
            </div>
          </div>

          {!isError ? (
            <>
              <div style={{ 
                background: 'rgba(34, 197, 94, 0.1)', 
                borderRadius: '8px', 
                padding: '16px',
                marginBottom: '16px',
                border: '1px solid rgba(34, 197, 94, 0.3)'
              }}>
                <div style={{ 
                  display: 'flex', 
                  alignItems: 'center', 
                  gap: '12px',
                  marginBottom: '8px'
                }}>
                  <CheckCircle size={20} style={{ color: '#22c55e', flexShrink: 0 }} />
                  <p style={{ margin: 0, fontSize: '15px', fontWeight: 600 }}>
                    Successfully added to library!
                  </p>
                </div>
              </div>

              <div style={{ 
                background: 'rgba(255, 255, 255, 0.05)', 
                borderRadius: '8px', 
                padding: '14px',
                border: '1px solid rgba(255, 255, 255, 0.1)'
              }}>
                <p style={{ 
                  margin: 0, 
                  fontSize: '14px', 
                  color: 'rgba(255, 255, 255, 0.7)',
                  wordBreak: 'break-word'
                }}>
                  <strong style={{ color: 'rgba(255, 255, 255, 0.9)' }}>Song:</strong> {songName}
                </p>
              </div>
            </>
          ) : (
            <div style={{ 
              background: 'rgba(239, 68, 68, 0.1)', 
              borderRadius: '8px', 
              padding: '16px',
              marginBottom: '16px',
              border: '1px solid rgba(239, 68, 68, 0.3)'
            }}>
              <p style={{ margin: '0 0 8px 0', fontSize: '15px', fontWeight: 600, color: '#ef4444' }}>
                Failed to upload song
              </p>
              <p style={{ 
                margin: 0, 
                fontSize: '14px', 
                color: 'rgba(255, 255, 255, 0.7)',
                wordBreak: 'break-word',
                fontFamily: 'monospace'
              }}>
                {errorMessage}
              </p>
            </div>
          )}
        </div>

        <div className="modal-footer" style={{ padding: '16px 24px', borderTop: '1px solid rgba(255, 255, 255, 0.1)' }}>
          <button 
            onClick={onClose}
            style={{
              width: '100%',
              padding: '10px 20px',
              borderRadius: '6px',
              background: isError ? 'rgba(239, 68, 68, 0.2)' : 'rgba(59, 130, 246, 0.2)',
              border: isError ? '1px solid rgba(239, 68, 68, 0.4)' : '1px solid rgba(59, 130, 246, 0.4)',
              color: 'white',
              fontSize: '14px',
              fontWeight: 600,
              cursor: 'pointer',
              transition: 'all 0.2s'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = isError ? 'rgba(239, 68, 68, 0.3)' : 'rgba(59, 130, 246, 0.3)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = isError ? 'rgba(239, 68, 68, 0.2)' : 'rgba(59, 130, 246, 0.2)';
            }}
          >
            OK
          </button>
        </div>
      </div>
    </div>
  );
}
