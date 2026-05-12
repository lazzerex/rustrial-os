package tools.src;

import haxe.io.Path;
import sys.FileSystem;
import sys.io.File;
import StringTools;

class Scaffold {
    public static function createScript(name:String, examplesDir:String):String {
        var trimmed = StringTools.trim(name);
        if (trimmed.length == 0) {
            Sys.println("Scaffold: script name is empty");
            Sys.exit(1);
        }

        if (trimmed.indexOf("/") != -1 || trimmed.indexOf("\\") != -1) {
            Sys.println("Scaffold: script name must not include path separators");
            Sys.exit(1);
        }

        var fileName = trimmed;
        if (!StringTools.endsWith(fileName, ".rscript")) {
            fileName = fileName + ".rscript";
        }

        var path = Path.join([examplesDir, fileName]);
        if (FileSystem.exists(path)) {
            Sys.println("Scaffold: file already exists: " + fileName);
            Sys.exit(1);
        }

        var template = "// " + fileName + "\n" +
            "let x = 0;\n" +
            "print(x);\n";

        File.saveContent(path, template);

        try {
            var tokens = Lexer.tokenize(template);
            Parser.parse(tokens);
        } catch (e) {
            if (FileSystem.exists(path)) {
                FileSystem.deleteFile(path);
            }
            Sys.println("Scaffold: " + Std.string(e));
            Sys.exit(1);
        }

        return path;
    }
}
