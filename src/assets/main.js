window.buffer = "";
window.alive = true;
window.exitCode = 0;
window.stdoutReadData = function () {
    if (window.stdoutAttached !== undefined)
        if (window.stdoutAttached && buffer.length > 0) {
            let tmp = window.buffer;
            window.buffer = "";
            return tmp;
        }
    return null;
};
const script = document.createElement("script");
script.type = "text/javascript";
script.src = "./third_party/hterm_all.js";
script.onload = async () => {
    await lib.init();

    const terminal = new hterm.Terminal();
    terminal.decorate(document.querySelector("#terminal"));
    terminal.prefs_.set("font-family", "'Fira Code', monospace");

    const jswasi = new Jswasi();
    await jswasi.attachDevice({terminal}, 1);
    await jswasi.init();
};
document.body.appendChild(script);
