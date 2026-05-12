package tools.src;

import haxe.io.Path;
import sys.FileSystem;
import sys.io.File;
import StringTools;

class Pipeline {
    public static function main():Void {
        var args = Sys.args();
        var filtered = args.filter(function(arg) return arg != "--");
        var includeAll = false;
        var excludes = new Map<String, Bool>();
        var newScriptName:String = null;

        if (filtered.length > 0 && filtered[0] == "new-script") {
            if (filtered.length < 2) {
                Sys.println("Pipeline: missing script name after 'new-script'");
                Sys.exit(1);
            }
            newScriptName = filtered[1];
            filtered = filtered.slice(2);
        }

        var i = 0;
        while (i < filtered.length) {
            var arg = filtered[i];
            if (arg == "--include-all") {
                includeAll = true;
                i++;
                continue;
            }
            if (StringTools.startsWith(arg, "--exclude=")) {
                addExclude(excludes, arg.substr("--exclude=".length));
                i++;
                continue;
            }
            if (arg == "--exclude") {
                if (i + 1 >= filtered.length) {
                    Sys.println("Pipeline: missing value for --exclude");
                    Sys.exit(1);
                }
                addExclude(excludes, filtered[i + 1]);
                i += 2;
                continue;
            }

            Sys.println("Pipeline: unknown argument '" + arg + "'");
            Sys.exit(1);
        }

        var examplesDir = findExamplesDir();
        if (examplesDir == null) {
            Sys.println("Pipeline: examples dir not found");
            Sys.exit(1);
        }

        if (newScriptName != null) {
            var created = Scaffold.createScript(newScriptName, examplesDir);
            Sys.println("Pipeline: created " + Path.withoutDirectory(created));
        }

        var files = FileSystem.readDirectory(examplesDir)
            .filter(function(name) return StringTools.endsWith(name, ".rscript"));
        files.sort(Reflect.compare);

        if (files.length == 0) {
            Sys.println("Pipeline: no .rscript files found");
            Sys.exit(1);
        }

        var failures = 0;
        for (file in files) {
            var path = Path.join([examplesDir, file]);
            var source = File.getContent(path);
            try {
                var tokens = Lexer.tokenize(source);
                Parser.parse(tokens);
            } catch (e) {
                var info = parseError(Std.string(e));
                printError(file, info.line, info.msg);
                failures++;
            }
        }

        if (failures > 0) {
            Sys.exit(1);
        }

        var loaderPath = findScriptLoader();
        if (loaderPath == null) {
            Sys.println("Pipeline: src/script_loader.rs not found");
            Sys.exit(1);
        }

        var oldContent = File.getContent(loaderPath);
        var oldLines = oldContent.split("\n");

        var existingOrder = new Array<String>();
        var commented = new Map<String, Bool>();
        var seen = new Map<String, Bool>();
        for (line in oldLines) {
            var trimmed = StringTools.trim(line);
            var isComment = StringTools.startsWith(trimmed, "//");
            if (isComment) {
                trimmed = StringTools.trim(trimmed.substr(2));
            }

            var name = parseScriptName(trimmed);
            if (name != null) {
                commented.set(name, isComment);
                if (!seen.exists(name)) {
                    seen.set(name, true);
                    existingOrder.push(name);
                }
            }
        }

        var available = new Map<String, Bool>();
        for (file in files) {
            available.set(file, true);
        }

        var finalOrder = new Array<String>();
        var used = new Map<String, Bool>();
        for (name in existingOrder) {
            if (available.exists(name)) {
                finalOrder.push(name);
                used.set(name, true);
            }
        }

        for (file in files) {
            if (!used.exists(file)) {
                finalOrder.push(file);
            }
        }

        var entries = new Array<String>();
        for (name in finalOrder) {
            var entry = "(\"" + name + "\", include_bytes!(\"rustrial_script/examples/" + name + "\")),";
            var isExcluded = excludes.exists(name);
            var wasCommented = commented.exists(name) && commented.get(name);
            var commentOut = isExcluded || (!includeAll && wasCommented);
            if (commentOut) {
                entries.push("    // " + entry);
            } else {
                entries.push("    " + entry);
            }
        }

        var range = findScriptsRange(oldLines);
        if (range == null) {
            Sys.println("Pipeline: could not locate SCRIPTS array in script_loader.rs");
            Sys.exit(1);
        }

        var before = oldLines.slice(0, range.start + 1);
        var after = oldLines.slice(range.end, oldLines.length);
        var newLines = before.concat(entries).concat(after);
        var newContent = newLines.join("\n");

        if (oldContent == newContent) {
            Sys.println("Pipeline: no changes");
            Sys.exit(0);
        }

        printDiff(oldLines, newLines);
        File.saveContent(loaderPath, newContent);
        Sys.println("Pipeline: wrote " + Path.withoutDirectory(loaderPath));
    }

    static function addExclude(excludes:Map<String, Bool>, name:String):Void {
        var trimmed = StringTools.trim(name);
        if (trimmed.length == 0) {
            return;
        }
        if (!StringTools.endsWith(trimmed, ".rscript")) {
            trimmed = trimmed + ".rscript";
        }
        excludes.set(trimmed, true);
    }

    static function parseScriptName(line:String):String {
        if (line.indexOf("include_bytes!(\"rustrial_script/examples/") == -1) {
            return null;
        }

        var start = line.indexOf("(\"");
        var end = line.indexOf("\", include_bytes!");
        if (start == -1 || end == -1 || end <= start + 2) {
            return null;
        }

        return line.substr(start + 2, end - (start + 2));
    }

    static function findScriptsRange(lines:Array<String>):{ start:Int, end:Int } {
        var start = -1;
        var end = -1;
        for (i in 0...lines.length) {
            if (lines[i].indexOf("pub const SCRIPTS") != -1) {
                start = i;
                break;
            }
        }
        if (start == -1) {
            return null;
        }

        for (i in start + 1...lines.length) {
            if (StringTools.trim(lines[i]) == "];" || StringTools.trim(lines[i]) == "];\r") {
                end = i;
                break;
            }
        }

        if (end == -1) {
            return null;
        }

        return { start: start, end: end };
    }

    static function printDiff(oldLines:Array<String>, newLines:Array<String>):Void {
        Sys.println("Pipeline diff:");
        var max = oldLines.length > newLines.length ? oldLines.length : newLines.length;
        for (i in 0...max) {
            var oldLine = i < oldLines.length ? oldLines[i] : null;
            var newLine = i < newLines.length ? newLines[i] : null;
            if (oldLine == newLine) {
                continue;
            }
            if (oldLine != null) {
                Sys.println("- " + oldLine);
            }
            if (newLine != null) {
                Sys.println("+ " + newLine);
            }
        }
    }

    static function parseError(err:String):{ line:Int, msg:String } {
        if (StringTools.startsWith(err, "Line ")) {
            var colon = err.indexOf(":");
            if (colon > 5) {
                var lineStr = err.substr(5, colon - 5);
                var line = Std.parseInt(lineStr);
                var msg = StringTools.trim(err.substr(colon + 1));
                if (line != null) {
                    return { line: line, msg: msg };
                }
            }
        }

        return { line: -1, msg: err };
    }

    static function printError(file:String, line:Int, msg:String):Void {
        if (line > 0) {
            Sys.println(file + ":" + line + ": " + msg);
        } else {
            Sys.println(file + ": " + msg);
        }
    }

    static function findExamplesDir():String {
        var cwd = Sys.getCwd();
        var candidates = [
            Path.normalize(Path.join([cwd, "src", "rustrial_script", "examples"])),
            Path.normalize(Path.join([cwd, "..", "src", "rustrial_script", "examples"]))
        ];

        for (path in candidates) {
            if (FileSystem.exists(path) && FileSystem.isDirectory(path)) {
                return path;
            }
        }

        return null;
    }

    static function findScriptLoader():String {
        var cwd = Sys.getCwd();
        var candidates = [
            Path.normalize(Path.join([cwd, "src", "script_loader.rs"])),
            Path.normalize(Path.join([cwd, "..", "src", "script_loader.rs"]))
        ];

        for (path in candidates) {
            if (FileSystem.exists(path) && !FileSystem.isDirectory(path)) {
                return path;
            }
        }

        return null;
    }
}
