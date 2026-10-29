; Inno Setup 6 Script
; PESU WiFi Windows Setup (.exe Installer)

#ifndef AppVersion
#define AppVersion "3.7.6"
#endif

#define AppName "PESU WiFi"
#define AppPublisher "imyash0722"
#define AppURL "https://github.com/imyash0722/pesu-wifi"
#define AppExeName "pesu-wifi.exe"

[Setup]
AppId={{D37D3E5E-5D88-4F29-9F83-6C78921B59C2}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}
DefaultDirName={localappdata}\Programs\pesu-wifi
DefaultGroupName={#AppName}
AllowNoIcons=yes
LicenseFile=LICENSE
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=dist
OutputBaseFilename=pesu-wifi-v{#AppVersion}-windows-x86_64-setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ChangesEnvironment=yes
UninstallDisplayIcon={app}\{#AppExeName}
UninstallDisplayName={#AppName} Login Manager & Watchdog

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "addtopath"; Description: "Add pesu-wifi to your user PATH environment variable"; GroupDescription: "System Environment:"; Flags: checkedonce
Name: "autostart"; Description: "Start continuous keepalive watchdog automatically at Windows logon"; GroupDescription: "Background Daemon:"; Flags: checkedonce
Name: "desktopicon"; Description: "Create a desktop shortcut for PESU WiFi Status"; GroupDescription: "Additional Shortcuts:"; Flags: unchecked

[Files]
Source: "target\release\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "install.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{userprograms}\{#AppName}\{#AppName} Status"; Filename: "{cmd}"; Parameters: "/k """"{app}\{#AppExeName}"""" status"; IconFilename: "{app}\{#AppExeName}"
Name: "{userprograms}\{#AppName}\Configure Credentials"; Filename: "{cmd}"; Parameters: "/k """"{app}\{#AppExeName}"""" add"; IconFilename: "{app}\{#AppExeName}"
Name: "{userprograms}\{#AppName}\Start Watchdog Daemon"; Filename: "{cmd}"; Parameters: "/c """"{app}\{#AppExeName}"""" start"; IconFilename: "{app}\{#AppExeName}"
Name: "{userprograms}\{#AppName}\Stop Watchdog Daemon"; Filename: "{cmd}"; Parameters: "/c """"{app}\{#AppExeName}"""" stop"; IconFilename: "{app}\{#AppExeName}"
Name: "{userprograms}\{#AppName}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{userdesktop}\{#AppName} Status"; Filename: "{cmd}"; Parameters: "/k """"{app}\{#AppExeName}"""" status"; IconFilename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "PESU-WiFi"; ValueData: """{app}\{#AppExeName}"" daemon"; Tasks: autostart; Flags: uninsdeletevalue

[Run]
Filename: "{cmd}"; Parameters: "/c """"{app}\{#AppExeName}"""" status & timeout /t 6"; Description: "Check campus Wi-Fi & gateway status"; Flags: postinstall skipifsilent nowait

[UninstallRun]
Filename: "{app}\{#AppExeName}"; Parameters: "stop"; Flags: runhidden

[Code]
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + UpperCase(Param) + ';', ';' + UpperCase(OrigPath) + ';') = 0;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  AppDir, OrigPath, NewPath: string;
begin
  if (CurStep = ssPostInstall) and IsTaskSelected('addtopath') then
  begin
    AppDir := ExpandConstant('{app}');
    if RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
    begin
      if NeedsAddPath(AppDir) then
      begin
        if (Length(OrigPath) > 0) and (OrigPath[Length(OrigPath)] <> ';') then
          NewPath := OrigPath + ';' + AppDir
        else
          NewPath := OrigPath + AppDir;
        RegWriteStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', NewPath);
      end;
    end
    else
    begin
      RegWriteStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', AppDir);
    end;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  AppDir, OrigPath, NewPath: string;
  P: Integer;
begin
  if CurUninstallStep = usUninstall then
  begin
    AppDir := ExpandConstant('{app}');
    if RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
    begin
      NewPath := OrigPath;
      P := Pos(';' + AppDir, NewPath);
      if P > 0 then
        Delete(NewPath, P, Length(AppDir) + 1)
      else
      begin
        P := Pos(AppDir + ';', NewPath);
        if P > 0 then
          Delete(NewPath, P, Length(AppDir) + 1)
        else
        begin
          P := Pos(AppDir, NewPath);
          if P > 0 then
            Delete(NewPath, P, Length(AppDir));
        end;
      end;
      RegWriteStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', NewPath);
    end;
  end;
end;
