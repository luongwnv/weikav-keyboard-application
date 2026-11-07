#ifndef AppVersion
#define AppVersion "0.9.4"
#endif
#ifndef SourceDir
#define SourceDir "..\dist\Weikav-Studio-Windows"
#endif
#ifndef OutputDir
#define OutputDir "..\dist\release"
#endif
[Setup]
AppId={{A4FAF0CE-C64E-43BC-9F54-5177DAE30D75}
AppName=Weikav Studio
AppVersion={#AppVersion}
AppPublisher=luongwnv
AppPublisherURL=https://luongwnv.github.io/weikav-app/
AppSupportURL=https://github.com/luongwnv/Weikav-Keyboard-Application/issues
DefaultDirName={localappdata}\Programs\Weikav Studio
DefaultGroupName=Weikav Studio
UninstallDisplayIcon={app}\Weikav Studio.exe
OutputDir={#OutputDir}
OutputBaseFilename=Weikav-Studio-{#AppVersion}-Windows-x64-Setup
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
Name: "{group}\Weikav Studio"; Filename: "{app}\Weikav Studio.exe"
[Run]
Filename: "{app}\Weikav Studio.exe"; Description: "Open Weikav Studio"; Flags: nowait postinstall skipifsilent
