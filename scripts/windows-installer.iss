#ifndef AppVersion
#define AppVersion "0.9.0"
#endif
#ifndef SourceDir
#define SourceDir "..\dist\NUT-Studio-Windows"
#endif
#ifndef OutputDir
#define OutputDir "..\dist\release"
#endif
[Setup]
AppId={{A4FAF0CE-C64E-43BC-9F54-5177DAE30D75}
AppName=NUT Studio
AppVersion={#AppVersion}
AppPublisher=luongwnv
AppPublisherURL=https://luongwnv.github.io/weikav-app/
AppSupportURL=https://github.com/luongwnv/Weikav-Keyboard-Application/issues
DefaultDirName={localappdata}\Programs\NUT Studio
DefaultGroupName=NUT Studio
UninstallDisplayIcon={app}\NUT Studio.exe
OutputDir={#OutputDir}
OutputBaseFilename=NUT-Studio-{#AppVersion}-Windows-x64-Setup
SetupIconFile=..\assets\app-icon\icon.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
DisableProgramGroupPage=yes
[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
[Icons]
Name: "{group}\NUT Studio"; Filename: "{app}\NUT Studio.exe"
[Run]
Filename: "{app}\NUT Studio.exe"; Description: "Open NUT Studio"; Flags: nowait postinstall skipifsilent
