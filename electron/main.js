const { app, BrowserWindow } = require('electron');

let mainWindow;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1180,
    height: 860,
    minWidth: 800,
    minHeight: 600,
    title: 'Lighthouse',
    titleBarStyle: 'hiddenInset',
    backgroundColor: '#03070b',
    show: false,
    webPreferences: {
      contextIsolation: false,
      nodeIntegration: true,
    },
  });

  mainWindow.loadURL('http://localhost:5189');

  // Show only once the page is painted to avoid a white flash, then pull the
  // window to the foreground.
  mainWindow.once('ready-to-show', () => {
    mainWindow.show();
    mainWindow.focus();
  });
}

app.whenReady().then(() => {
  // macOS: when Electron is launched from a terminal / npm script it can come
  // up as an accessory app, leaving the window hidden behind everything (no
  // dock icon, never focused). Forcing the regular activation policy
  // (NSApplicationActivationPolicyRegular) + showing the dock + stealing focus
  // makes the window reliably appear.
  if (process.platform === 'darwin') {
    app.setActivationPolicy('regular');
    if (app.dock) app.dock.show();
  }
  createWindow();
  app.focus({ steal: true });
});

app.on('window-all-closed', () => { if (process.platform !== 'darwin') app.quit(); });
app.on('activate', () => { if (BrowserWindow.getAllWindows().length === 0) createWindow(); });
