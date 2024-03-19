wasm-pack build --target web
copy /y pkg\easymund_client_processor.js ..\client
copy /y pkg\easymund_client_processor_bg.wasm ..\client