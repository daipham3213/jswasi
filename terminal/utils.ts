import * as constants from "./constants";

export function arraysEqual(a: any[], b: any[]) {
    if (a === b) return true;
    if (a == null || b == null) return false;
    if (a.length !== b.length) return false;

    for (let i = 0; i < a.length; ++i) {
        if (a[i] !== b[i]) return false;
    }

    return true;
}

export function fix_path(path: string, env) {
    // TODO: home handling should be moved to shell in the end
    if (path == "~") return env['HOME'];
    if (path == "/~") return env['HOME'];
    if (path.substr(0,2) == "~/") return env['HOME'] + path.substr(2,4096);
    // if (path[0] == ".") if (env['PWD'] != "/") return path.replace(".", env['PWD']);
    return path;
    let pwd = env['PWD'];
    if (pwd != "/") pwd = pwd + "/";
    if (path.length == 0) return path;
    if (path[0] == '!') return path;
    if (path[0] == '/') return path;
    if (path[0] != '.') return "/" + path;
    if (path.substr(0,2) == "./") return pwd + path.substr(2);
    return pwd + path;
}

export function realpath(path, env) {
    let result = [];
    let result_path = "";
    let tmp_path = path;
    let part = "";
    let level = 0;
    let root_path = (path[0] == '/');
    while (tmp_path != "") {
        if (tmp_path.indexOf("/") != -1) {
            part = tmp_path.substr(0, tmp_path.indexOf("/"));
        } else part = tmp_path;
        tmp_path = tmp_path.substr(part.length+1);
        if (part == "..") {
            if (level > 0) level -= 1;
        } else if (part == "~") {
            // TODO: shell should always parse this in the end and provide "normal" paths
            result[level] = env["HOME"];
            level++;
        } else {
            result[level] = part;
            level++;
        }
    }
    result_path = result.slice(0, level).join("/");
    if (root_path) if (result_path == "") return "/";
    return result_path;
}
