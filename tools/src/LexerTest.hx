package tools.src;

import haxe.io.Path;
import sys.FileSystem;
import sys.io.File;
import StringTools;
import tools.src.Lexer.Token;
import tools.src.Lexer.TokenInfo;

class LexerTest {
    public static function main():Void {
        var examplesDir = findExamplesDir();
        if (examplesDir == null) {
            Sys.println("LexerTest: examples dir not found");
            Sys.exit(1);
        }

        var files = FileSystem.readDirectory(examplesDir)
            .filter(function(name) return StringTools.endsWith(name, ".rscript"));
        files.sort(Reflect.compare);

        if (files.length == 0) {
            Sys.println("LexerTest: no .rscript files found");
            Sys.exit(1);
        }

        var failures = 0;
        for (file in files) {
            var path = Path.join([examplesDir, file]);
            var source = File.getContent(path);
            try {
                var tokens = Lexer.tokenize(source);
                if (tokens.length == 0) {
                    reportFail(file, "no tokens");
                    failures++;
                    continue;
                }

                if (!isEof(tokens[tokens.length - 1].kind)) {
                    reportFail(file, "missing TEof");
                    failures++;
                    continue;
                }

                if (!linesNonDecreasing(tokens)) {
                    reportFail(file, "line numbers not monotonic");
                    failures++;
                    continue;
                }
            } catch (e) {
                reportFail(file, Std.string(e));
                failures++;
            }
        }

        if (failures > 0) {
            Sys.println("LexerTest: failed " + failures + " file(s)");
            Sys.exit(1);
        }

        Sys.println("LexerTest: ok (" + files.length + " file(s))");
    }

    static function isEof(kind:Token):Bool {
        return switch (kind) {
            case TEof: true;
            default: false;
        };
    }

    static function linesNonDecreasing(tokens:Array<TokenInfo>):Bool {
        var last = 1;
        for (t in tokens) {
            if (t.line < last) {
                return false;
            }
            last = t.line;
        }
        return true;
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
