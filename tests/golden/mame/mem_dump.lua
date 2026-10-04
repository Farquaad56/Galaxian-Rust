local DUMPS = {[60]=true, [600]=true}
local LAST  = 600
local scr = manager.machine.screens[":screen"]
local mem = manager.machine.devices[":maincpu"].spaces["program"]
local function dump(path)
  local t = {}
  for a = 0x5000, 0x53FF do t[#t+1] = string.char(mem:read_u8(a)) end   -- VRAM : 0x400 octets
  for a = 0x5800, 0x58FF do t[#t+1] = string.char(mem:read_u8(a)) end   -- OBJRAM : 0x100 octets
  local f = assert(io.open(path, "wb")); f:write(table.concat(t)); f:close()
end
local function on_frame()
  local n = scr:frame_number()
  if DUMPS[n] then dump(string.format("../tests/golden/mem_dump_%d.bin", n)) end
  if n >= LAST then manager.machine:exit() end
end
emu.register_frame_done(on_frame, "frame")
