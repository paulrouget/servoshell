<?xml version="1.0" encoding="utf-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*"
           Name="ServoShell"
           Manufacturer="Mozilla Research"
           UpgradeCode="74ef71e5-14e7-4139-9615-5c138edfb5bf"
           Language="1033"
           Codepage="1252"
           Version="1.0.0">
    <Package Id="*"
             Keywords="Installer"
             Description="ServoShell Installer"
             Manufacturer="Mozilla Research"
             InstallerVersion="200"
             Platform="x64"
             Languages="1033"
             SummaryCodepage="1252"
             Compressed="yes"/>
    <MajorUpgrade AllowDowngrades="yes"/>
    <Media Id="1"
           Cabinet="ServoShell.cab"
           EmbedCab="yes"/>
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFiles64Folder" Name="PFiles">
        <Directory Id="MozResearch" Name="Mozilla Research">
          <Directory Id="INSTALLDIR" Name="ServoShell">
            <Component Id="ServoShell"
                       Guid="9f135ad0-1cd2-482c-bac8-05e04c675ba4"
                       Win64="yes">
              <File Id="ServoShellEXE"
                    Name="servoshell.exe"
                    DiskId="1"
                    Source="${windowize(exe_path)}\servoshell.exe"
                    KeyPath="yes">
                <Shortcut Id="StartMenuServoShell"
                          Directory="ProgramMenuDir"
                          Name="ServoShell"
                          WorkingDirectory="INSTALLDIR"
                          Icon="ServoShell.ico"
                          IconIndex="0"
                          Advertise="yes"/>
              </File>
              ${include_dependencies()}
            </Component>

            ${include_directory(resources_path, "resources")}
          </Directory>
        </Directory>
      </Directory>

      <Directory Id="ProgramMenuFolder" Name="Programs">
        <Directory Id="ProgramMenuDir" Name="ServoShell">
          <Component Id="ProgramMenuDir" Guid="2834f3a8-9441-4c3e-bcd2-14cd437c474e">
            <RemoveFolder Id="ProgramMenuDir" On="both"/>
            <RegistryValue Root="HKCU"
                           Key="Software\Mozilla Research\ServoShell"
                           Type="string"
                           Value=""
                           KeyPath="yes"/>
          </Component>
        </Directory>
      </Directory>
    </Directory>

    <Feature Id="Complete" Level="1">
      <ComponentRef Id="ServoShell"/>
      % for c in components:
      <ComponentRef Id="${c}"/>
      % endfor
      <ComponentRef Id="ProgramMenuDir"/>
    </Feature>

    <Icon Id="ServoShell.ico" SourceFile="${windowize(resources_path)}\shell_resources\icons\ServoShell.ico"/>
  </Product>
</Wix>
<%!
import os
import os.path as path
import re
import uuid
from servo.command_base import host_triple

def make_id(s):
    s = s.replace("-", "_").replace("/", "_").replace("\\", "_")
    return "Id{}".format(s)

def listfiles(directory):
    return [f for f in os.listdir(directory)
            if path.isfile(path.join(directory, f))]

def listdirs(directory):
    return [f for f in os.listdir(directory)
            if path.isdir(path.join(directory, f))]

def listdeps(temp_dir):
    return [path.join(temp_dir, f) for f in os.listdir(temp_dir) if os.path.isfile(path.join(temp_dir, f)) and f != "servoshell.exe"]

def windowize(p):
    if not p.startswith("/"):
        return p
    return re.sub("^/([^/])+", "\\1:", p)

components = []
%>

<%def name="include_dependencies()">
% for f in listdeps(dir_to_temp):
              <File Id="${make_id(path.basename(f)).replace(".","").replace("+","x")}"
                    Name="${path.basename(f)}"
                    Source="${f}"
                    DiskId="1"/>
% endfor
</%def>

<%def name="include_directory(d, n)">
<Directory Id="${make_id(path.basename(d))}" Name="${n}">
  <Component Id="${make_id(path.basename(d))}"
             Guid="${uuid.uuid4()}"
             Win64="yes">
    <CreateFolder/>
    <% components.append(make_id(path.basename(d))) %>
    % for f in listfiles(d):
    <File Id="${make_id(path.join(d, f).replace(dir_to_temp, ""))}"
          Name="${f}"
          Source="${windowize(path.join(d, f))}"
          DiskId="1"/>
    % endfor
  </Component>

  % for f in listdirs(d):
  ${include_directory(path.join(d, f), f)}
  % endfor
</Directory>
</%def>
