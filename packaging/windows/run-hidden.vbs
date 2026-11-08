Set WshShell = CreateObject("WScript.Shell")
Set FSO = CreateObject("Scripting.FileSystemObject")

If WScript.Arguments.Count > 0 Then
    ExePath = WScript.Arguments(0)
    AppDir = FSO.GetParentFolderName(ExePath)
    WshShell.CurrentDirectory = AppDir
    
    Args = ""
    For i = 1 To WScript.Arguments.Count - 1
        Args = Args & " " & WScript.Arguments(i)
    Next
    
    If Trim(Args) = "" Then
        CmdLine = """" & ExePath & """ --no-browser"
    Else
        CmdLine = """" & ExePath & """" & Args
    End If
    
    WshShell.Run CmdLine, 0, False
End If
