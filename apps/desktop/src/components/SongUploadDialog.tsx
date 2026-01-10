import { X, CheckCircle, XCircle } from "lucide-react";

interface UploadResult {
  songName: string;
  isError: boolean;
  errorMessage?: string;
}

interface SongUploadDialogProps {
  onClose: () => void;
  results: UploadResult[];
  isMultipleUpload?: boolean;
}

export default function SongUploadDialog({ 
  onClose, 
  results,
  isMultipleUpload = false
}: SongUploadDialogProps) {
  const successCount = results.filter(r => !r.isError).length;
  const errorCount = results.filter(r => r.isError).length;
  const hasErrors = errorCount > 0;
  const hasSuccesses = successCount > 0;
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()} style={{ maxWidth: '450px' }}>
        <div className="modal-header">
          <h2 style={{ margin: 0, fontSize: '22px', fontWeight: 700 }}>
            {isMultipleUpload ? 
              (hasErrors && hasSuccesses ? 'Partial Upload Success' : 
               hasErrors ? 'Upload Failed' : 'Songs Uploaded') :
              (hasErrors ? 'Upload Failed' : 'Song Uploaded')
            }
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
              background: hasErrors && hasSuccesses ? 'rgba(249, 115, 22, 0.15)' :
                          hasErrors ? 'rgba(239, 68, 68, 0.15)' : 'rgba(34, 197, 94, 0.15)',
              marginBottom: '12px'
            }}>
              {hasErrors && hasSuccesses ? (
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
                  <circle cx="12" cy="12" r="9" stroke="#f97316" strokeWidth="2" fill="none" />
                  <path d="M12 8v4" stroke="#f97316" strokeWidth="2" strokeLinecap="round" />
                  <circle cx="12" cy="16" r="1" fill="#f97316" />
                </svg>
              ) : hasErrors ? (
                <XCircle size={48} style={{ color: '#ef4444' }} />
              ) : (
                <CheckCircle size={48} style={{ color: '#22c55e' }} />
              )}
            </div>
          </div>

          {/* Summary section */}
          {isMultipleUpload && (
            <div style={{ marginBottom: '16px' }}>
              <div style={{
                display: 'grid',
                gridTemplateColumns: '1fr 1fr',
                gap: '8px',
                marginBottom: '12px'
              }}>
                <div style={{
                  background: 'rgba(34, 197, 94, 0.1)',
                  border: '1px solid rgba(34, 197, 94, 0.3)',
                  borderRadius: '6px',
                  padding: '8px 12px',
                  textAlign: 'center'
                }}>
                  <div style={{ fontSize: '18px', fontWeight: 'bold', color: '#22c55e' }}>{successCount}</div>
                  <div style={{ fontSize: '12px', color: 'rgba(255, 255, 255, 0.7)' }}>Successful</div>
                </div>
                <div style={{
                  background: 'rgba(239, 68, 68, 0.1)',
                  border: '1px solid rgba(239, 68, 68, 0.3)',
                  borderRadius: '6px',
                  padding: '8px 12px',
                  textAlign: 'center'
                }}>
                  <div style={{ fontSize: '18px', fontWeight: 'bold', color: '#ef4444' }}>{errorCount}</div>
                  <div style={{ fontSize: '12px', color: 'rgba(255, 255, 255, 0.7)' }}>Failed</div>
                </div>
              </div>
            </div>
          )}

          {/* Results list */}
          <div style={{
            maxHeight: '300px',
            overflowY: 'auto',
            marginBottom: '16px'
          }}>
            {results.map((result, index) => (
              <div key={index} style={{
                background: 'rgba(255, 255, 255, 0.05)',
                borderRadius: '6px',
                padding: '12px',
                marginBottom: '8px',
                border: `1px solid ${result.isError ? 'rgba(239, 68, 68, 0.3)' : 'rgba(34, 197, 94, 0.3)'}`
              }}>
                <div style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  marginBottom: result.isError ? '4px' : '0'
                }}>
                  {result.isError ? (
                    <XCircle size={16} style={{ color: '#ef4444', flexShrink: 0 }} />
                  ) : (
                    <CheckCircle size={16} style={{ color: '#22c55e', flexShrink: 0 }} />
                  )}
                  <span style={{
                    fontSize: '14px',
                    fontWeight: 500,
                    color: result.isError ? '#ef4444' : '#22c55e',
                    wordBreak: 'break-word'
                  }}>
                    {result.songName}
                  </span>
                </div>
                {result.isError && result.errorMessage && (
                  <div style={{
                    fontSize: '12px',
                    color: 'rgba(255, 255, 255, 0.6)',
                    marginLeft: '24px',
                    wordBreak: 'break-word'
                  }}>
                    {result.errorMessage}
                  </div>
                )}
              </div>
            ))}
          </div>

          {/* Success message for single upload */}
          {!isMultipleUpload && hasSuccesses && (
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
          )}
        </div>

        <div className="modal-footer" style={{ padding: '16px 24px', borderTop: '1px solid rgba(255, 255, 255, 0.1)' }}>
          <button 
            onClick={onClose}
            style={{
              width: '100%',
              padding: '10px 20px',
              borderRadius: '6px',
              background: hasErrors ? 'rgba(239, 68, 68, 0.2)' : 'rgba(59, 130, 246, 0.2)',
              border: hasErrors ? '1px solid rgba(239, 68, 68, 0.4)' : '1px solid rgba(59, 130, 246, 0.4)',
              color: 'white',
              fontSize: '14px',
              fontWeight: 600,
              cursor: 'pointer',
              transition: 'all 0.2s'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = hasErrors ? 'rgba(239, 68, 68, 0.3)' : 'rgba(59, 130, 246, 0.3)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = hasErrors ? 'rgba(239, 68, 68, 0.2)' : 'rgba(59, 130, 246, 0.2)';
            }}
          >
            OK
          </button>
        </div>
      </div>
    </div>
  );
}
