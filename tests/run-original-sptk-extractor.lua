function try_execute(command)
  local ok, kind, code = os.execute(command)
  assert(ok, command .. " failed: " .. tostring(kind) .. " " .. tostring(code))
end
local extractor = assert(loadfile(arg[1]))()
extractor(try_execute, arg[2], arg[3], "")
