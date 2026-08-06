import re
with open("Core-Engine/thunder-rpc/src/server.rs", "r") as f:
    text = f.read()
pattern = re.compile(r'data_dir:\s*&format!\(.*?\.to_string\(\),', re.DOTALL)
replacement = 'data_dir: Box::leak(format!("/tmp/thunder_test_node_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()).into_boxed_str()),'
new_text = pattern.sub(replacement, text)
with open("Core-Engine/thunder-rpc/src/server.rs", "w") as f:
    f.write(new_text)
