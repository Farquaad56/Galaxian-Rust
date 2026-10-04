local OUT  = "../tests/golden/frames_crc.txt"
local WANT = {1,2,5,10,30,60,120,300,600}
local LAST = 600
local want = {}
for _, n in ipairs(WANT) do want[n] = true end
local tbl = {}
for i = 0, 255 do
  local c = i
  for _ = 1, 8 do
    if c & 1 == 1 then c = (c >> 1) ~ 0xEDB88320 else c = c >> 1 end
  end
  tbl[i] = c
end
local function crc32(s)
  local c = 0xFFFFFFFF
  for i = 1, #s do c = tbl[(c ~ s:byte(i)) & 0xFF] ~ (c >> 8) end
  return c ~ 0xFFFFFFFF
end
local scr = manager.machine.screens[":screen"]
local out = assert(io.open(OUT, "w"))
local first = nil
local function on_frame()
  local n = scr:frame_number()
  if first == nil then
    first = n
    out:write(string.format("# first_frame_seen=%d size=%dx%d mame=%s\n", n, scr.width, scr.height, emu.app_version()))
  end
  if want[n] then
    out:write(string.format("%d %08X %dx%d\n", n, crc32(scr:pixels()), scr.width, scr.height))
    out:flush()
  end
  if n >= LAST then out:close(); manager.machine:exit() end
end
emu.register_frame_done(on_frame, "frame")
