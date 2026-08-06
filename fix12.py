import re

# Fix 1: server.rs End of E0308 issue
path_server = "Core-Engine/thunder-rpc/src/server.rs"
with open(path_server, "r") as f:
    text = f.read()

pattern1 = r'data_dir: Box::leak\(\s*format!\('
text = re.sub(pattern1, 'data_dir: format!(', text)
pattern2 = r'\.into_boxed_str\(\),\s*\)'
text = re.sub(pattern2, ',', text)

with open(path_server, "w") as f:
    f.write(text)

# Fix 2: main.rs clippy warnings
path_main = "Core-Engine/thunder-cli/src/main.rs"
with open(path_main, "r") as f:
    main_rs = f.read()

main_rs = main_rs.replace("if let Ok(_) = n.create_event() {", "if n.create_event().is_ok() {")
main_rs = main_rs.replace("hex::encode(block.hash())[0..10].to_string()", "&hex::encode(block.hash())[0..10]")

with open(path_main, "w") as f:
    f.write(main_rs)

