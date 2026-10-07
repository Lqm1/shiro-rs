local root = assert(arg[3])
package.path = root .. "/?.lua;" .. root .. "/external/?.lua;" .. package.path
local cli = require("cli-common")
local json = require("dkjson")
local entries = assert(cli.load_index_file(arg[1], "data/", {"left"}, {"right"}))
local output = assert(io.open(arg[2], "wb"))
assert(output:write(json.encode(entries, {indent = true})))
assert(output:close())
