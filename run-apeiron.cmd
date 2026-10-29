@echo off
rem Launch Apeiron Agents. Double-click this, or run it from a terminal.
rem Reads OPENAI_API_KEY from .env (git-ignored) so no sign-in screen appears.
title Apeiron Agents
cd /d "%~dp0"
set COLORTERM=truecolor

if exist ".env" (
  for /f "usebackq tokens=1,* delims==" %%A in (".env") do (
    if /i "%%A"=="OPENAI_API_KEY" set "OPENAI_API_KEY=%%B"
  )
)

if "%OPENAI_API_KEY%"=="" (
  echo [warn] No OPENAI_API_KEY found in .env - the sign-in screen will appear.
  echo.
)

"%~dp0codex-rs\target\debug\codex.exe" --no-daemon
echo.
echo [Apeiron Agents exited with code %ERRORLEVEL%]
pause
