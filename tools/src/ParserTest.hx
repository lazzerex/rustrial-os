package tools.src;

import haxe.io.Path;
import sys.FileSystem;
import sys.io.File;
import StringTools;

class ParserTest {
    public static function main():Void {
        var examplesDir = findExamplesDir();
        if (examplesDir == null) {
            Sys.println("ParserTest: examples dir not found");
            Sys.exit(1);
        }

        var files = FileSystem.readDirectory(examplesDir)
            .filter(function(name) return StringTools.endsWith(name, ".rscript"));
        files.sort(Reflect.compare);

        if (files.length == 0) {
            Sys.println("ParserTest: no .rscript files found");
            Sys.exit(1);
        }

        var failures = 0;
        for (file in files) {
            var path = Path.join([examplesDir, file]);
            var source = File.getContent(path);
            try {
                var tokens = Lexer.tokenize(source);
                var bytecode = Parser.parse(tokens);
                if (bytecode.length == 0) {
                    reportFail(file, "empty bytecode");
                    failures++;
                    continue;
                }
            } catch (e) {
                reportFail(file, Std.string(e));
                failures++;
            }
        }

        if (failures > 0) {
            Sys.println("ParserTest: failed " + failures + " file(s)");
            Sys.exit(1);
        }

        Sys.println("ParserTest: ok (" + files.length + " file(s))");
    }

    static function reportFail(file:String, msg:String):Void {
        Sys.println(file + ": " + msg);
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
