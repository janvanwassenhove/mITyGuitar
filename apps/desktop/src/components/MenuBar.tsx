import { useState } from "react";
import { Music2, HelpCircle, Guitar, Minus, Maximize2, X, Loader2, Settings, Library, Play, Radio } from "lucide-react";
import { Window } from '@tauri-apps/api/window';

interface MenuBarProps {
  onMenuAction: (action: string) => void;
  currentView?: string;
  isRescanningSoundFonts?: boolean;
}

export default function MenuBar({ onMenuAction, currentView, isRescanningSoundFonts }: MenuBarProps) {
  const [openMenu, setOpenMenu] = useState<string | null>(null);

  const getViewDisplayName = (view: string) => {
    switch (view) {
      case 'live': return 'Live View';
      case 'diagnostics': return 'Diagnostics';
      case 'profile-manager': return 'Device Manager';
      case 'song-library': return 'Song Library';
      default: return view;
    }
  };

  const handleMenuClick = (menu: string) => {
    setOpenMenu(openMenu === menu ? null : menu);
  };

  const handleAction = (action: string) => {
    setOpenMenu(null);
    onMenuAction(action);
  };

  const handleMinimize = async () => {
    try {
      console.log('Minimize clicked');
      const appWindow = Window.getCurrent();
      await appWindow.minimize();
      console.log('Window minimized');
    } catch (error) {
      console.error('Failed to minimize window:', error);
    }
  };

  const handleMaximize = async () => {
    try {
      console.log('Maximize clicked');
      const appWindow = Window.getCurrent();
      await appWindow.toggleMaximize();
      console.log('Window maximized');
    } catch (error) {
      console.error('Failed to maximize window:', error);
    }
  };

  const handleClose = async () => {
    try {
      console.log('Close clicked');
      const appWindow = Window.getCurrent();
      await appWindow.close();
      console.log('Window closed');
    } catch (error) {
      console.error('Failed to close window:', error);
    }
  };

  return (
    <nav className="menu-bar">
      <div className="menu-items">
        {/* App Branding */}
        <div className="app-branding">
          <Guitar size={22} strokeWidth={2.5} className="app-icon" />
          <span className="app-name">mITyGuitar</span>
        </div>
        
        <div className="menu-divider"></div>
        
        {/* Live View - Direct Access */}
        <div className="menu-item">
          <button 
            onClick={() => handleAction("view_live")} 
            title="Live View" 
            className="menu-icon-btn"
          >
            <Radio size={20} strokeWidth={2} />
          </button>
        </div>

        {/* Song Play - Direct Access */}
        <div className="menu-item">
          <button 
            onClick={() => handleAction("view_song_play")} 
            title="Song Play" 
            className="menu-icon-btn"
          >
            <Play size={20} strokeWidth={2} />
          </button>
        </div>

        <div className="menu-divider"></div>

        {/* Library Icon - Quick Access */}
        <div className="menu-item">
          <button 
            onClick={() => handleAction("view_song_library")} 
            title="Song Library" 
            className="menu-icon-btn"
          >
            <Library size={20} strokeWidth={2} />
          </button>
        </div>

      

        {/* Instruments Menu */}
        <div className="menu-item">
          <button onClick={() => handleMenuClick("instruments")} title="Instruments" className="menu-icon-btn">
            <Music2 size={20} strokeWidth={2} />
          </button>
          {openMenu === "instruments" && (
            <div className="menu-dropdown">
              <button 
                className="menu-dropdown-item" 
                onClick={() => handleAction("choose_virtual_instrument")}
              >
                Virtual Instruments
              </button>
              <button 
                className="menu-dropdown-item" 
                onClick={() => handleAction("choose_soundfont")}
              >
                Choose SoundFont
              </button>
              <button 
                className="menu-dropdown-item" 
                onClick={() => handleAction("rescan_soundfonts")}
                disabled={isRescanningSoundFonts}
              >
                {isRescanningSoundFonts ? (
                  <>
                    <Loader2 size={16} className="menu-item-icon spinning" />
                    <span>Rescanning...</span>
                  </>
                ) : (
                  "Rescan SoundFonts"
                )}
              </button>
            </div>
          )}
        </div>

        <div className="menu-divider"></div>

        {/* Settings Menu */}
        <div className="menu-item">
          <button onClick={() => handleMenuClick("settings")} title="Settings" className="menu-icon-btn">
            <Settings size={20} strokeWidth={2} />
          </button>
          {openMenu === "settings" && (
            <div className="menu-dropdown">
              <button
                className="menu-dropdown-item"
                onClick={() => handleAction("audio_settings")}
              >
                Audio Settings
              </button>
              <button
                className="menu-dropdown-item"
                onClick={() => handleAction("view_diagnostics")}
              >
                Diagnostics
              </button>
              <button
                className="menu-dropdown-item"
                onClick={() => handleAction("view_profile_manager")}
              >
                Device Manager
              </button>
            </div>
          )}
        </div>

        {/* Help Menu */}
        <div className="menu-item">
          <button onClick={() => handleMenuClick("help")} title="Help" className="menu-icon-btn">
            <HelpCircle size={20} strokeWidth={2} />
          </button>
          {openMenu === "help" && (
            <div className="menu-dropdown">
              <button 
                className="menu-dropdown-item" 
                onClick={() => handleAction("about")}
              >
                About mITyGuitar
              </button>
            </div>
          )}
        </div>
      </div>
      
      {/* Current View Title */}
      {currentView && (
        <div className="view-title">
          {getViewDisplayName(currentView)}
        </div>
      )}
      
      {/* Window Controls */}
      <div className="window-controls">
        <button 
          type="button"
          onClick={handleMinimize} 
          className="window-control-btn" 
          title="Minimize"
          aria-label="Minimize"
        >
          <Minus size={16} strokeWidth={2} />
        </button>
        <button 
          type="button"
          onClick={handleMaximize} 
          className="window-control-btn" 
          title="Maximize"
          aria-label="Maximize"
        >
          <Maximize2 size={16} strokeWidth={2} />
        </button>
        <button 
          type="button"
          onClick={handleClose} 
          className="window-control-btn close-btn" 
          title="Close"
          aria-label="Close"
        >
          <X size={16} strokeWidth={2} />
        </button>
      </div>
    </nav>
  );
}
