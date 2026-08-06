path = "Core-Engine/thunder-rpc/src/server.rs"
with open(path, "r") as f:
    text = f.read()

import re

# We find `data_dir:*` and replace it entirely!
pattern = r'data_dir:\s*&format!\(".*?thunder_test_node_.*?"[^)]*\)\.to_string\(\)'
replacement = 'data_dir: Box::leak(format!("/tmp/thunder_test_node_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()).into_boxed_str())'

new_text = re.sub(pattern, replacement, text)

# Also fix the blank space at state.rs line 326 ` \n` 
# I will just run `cargo fmt`
with open(path, "w") as f:
    f.write(new_text)

