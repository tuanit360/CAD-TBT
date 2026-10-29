; TBT360 CAD R65 Windows installer. Build from Rust binaries, never from reference EXE.
#define MyAppName "TBT360 CAD R65"
#define MyAppVersion "0.13.0"
#define MyAppPublisher "TBT Viet Nam"
[Setup]
AppId={{0B643D1A-795E-4C0A-A36A-4AF2C03D1E6F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\Programs\TBT360 CAD R65
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\cadcraft.exe
OutputDir=..\dist
OutputBaseFilename=TBT360_CAD_R65_Setup
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
DisableProgramGroupPage=yes
[Files]
Source: "..\SOURCE_CAD_R65\target\release\cadcraft.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\SOURCE_CAD_R65\target\release\cadcraft-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
[Icons]
Name: "{autoprograms}\TBT360 CAD R65"; Filename: "{app}\cadcraft.exe"
Name: "{autodesktop}\TBT360 CAD R65"; Filename: "{app}\cadcraft.exe"; Tasks: desktopicon
[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"
[Run]
Filename: "{app}\cadcraft.exe"; Description: "Launch TBT360 CAD R65"; Flags: nowait postinstall skipifsilent
