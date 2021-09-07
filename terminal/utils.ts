export function realpath(path) {
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
             } else {
                 result[level] = part;
                 level++;
             }
        }
        result_path = result.slice(0, level).join("/");
        if (root_path) if (result_path == "") return "/";
        return result_path;
}

export function arraysEqual(a: any[], b: any[]) {
    if (a === b) return true;
    if (a == null || b == null) return false;
    if (a.length !== b.length) return false;
  
    for (var i = 0; i < a.length; ++i) {
      if (a[i] !== b[i]) return false;
    }

    return true;
}