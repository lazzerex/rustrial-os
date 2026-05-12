package tools.src;

import haxe.io.Path;
import sys.FileSystem;
import sys.io.File;
import StringTools;

class Validator {
    public static function main():Void {
        var args = Sys.args();
        var files = new Array<String>();

        var filtered = args.filter(function(arg) return arg != "--");

        if (filtered.length == 0) {
            var examplesDir = findExamplesDir();
            if (examplesDir == null) {
                Sys.println("Validator: examples dir not found");
                Sys.exit(1);
            }

            files = FileSystem.readDirectory(examplesDir)
                .filter(function(name) return StringTools.endsWith(name, ".rscript"))
                .map(function(name) return Path.join([examplesDir, name]));
        } else {
            for (arg in filtered) {
                files.push(arg);
            }
        }

        if (files.length == 0) {
            Sys.println("Validator: no .rscript files found");
            Sys.exit(1);
        }

        var failures = 0;
        for (path in files) {
            if (!FileSystem.exists(path) || FileSystem.isDirectory(path)) {
                printError(path, -1, "File not found");
                failures++;
                continue;
            }

            var source = File.getContent(path);
            try {
                var tokens = Lexer.tokenize(source);
                Parser.parse(tokens);
            } catch (e) {
                var err = Std.string(e);
                var info = parseError(err);
                printError(path, info.line, info.msg);
                failures++;
            }
        }

        if (failures > 0) {
            Sys.exit(1);
        }

        Sys.println("Validator: ok (" + files.length + " file(s))");
        Sys.exit(0);
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

    static function printError(path:String, line:Int, msg:String):Void {
        var name = Path.withoutDirectory(path);
        if (line > 0) {
            Sys.println(name + ":" + line + ": " + msg);
        } else {
            Sys.println(name + ": " + msg);
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
}
