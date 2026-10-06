# Port the full public functionality

Reimplement the public functionality of all three upstream projects, including SHIRO's Lua tools and dependency algorithms that SHIRO does not call. Restricting the dependency ports to SHIRO's call graph would reduce initial work, but it would produce partial library ports and conflict with the requested complete reimplementation. External Lua extractor execution remains a separate compatibility decision.
