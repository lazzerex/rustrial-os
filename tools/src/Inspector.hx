package tools.src;

import haxe.ui.HaxeUIApp;
import haxe.ui.components.Label;
import haxe.ui.components.TextArea;
import haxe.ui.containers.HBox;
import haxe.ui.containers.ScrollView;
import haxe.ui.containers.VBox;
import haxe.ui.events.UIEvent;
import haxe.ui.events.MouseEvent;
import js.Browser;
import tools.src.Parser.OpCode;

class Inspector {
    static var sourceArea:TextArea;
    static var status:Label;
    static var opcodeScroll:ScrollView;
    static var opcodeBox:VBox;
    static var opcodeEntries:Array<Label> = [];
    static var opcodeLines:Array<Int> = [];
    static var selectedIndex:Int = -1;

    public static function main():Void {
        if (Browser.document == null || Browser.document.body == null) {
            Browser.window.addEventListener("DOMContentLoaded", function(_e) {
                startApp();
            });
            return;
        }

        startApp();
    }

    static function startApp():Void {
        var app = new HaxeUIApp();
        app.ready(function() {
            var root = new HBox();
            root.percentWidth = 100;
            root.percentHeight = 100;

            var left = new VBox();
            left.percentWidth = 60;
            left.percentHeight = 100;

            var right = new VBox();
            right.percentWidth = 40;
            right.percentHeight = 100;

            var leftTitle = new Label();
            leftTitle.text = "Source";

            sourceArea = new TextArea();
            sourceArea.text = defaultSource();
            sourceArea.percentWidth = 100;
            sourceArea.percentHeight = 100;

            var rightTitle = new Label();
            rightTitle.text = "Bytecode";

            opcodeScroll = new ScrollView();
            opcodeScroll.percentWidth = 100;
            opcodeScroll.percentHeight = 100;
            Reflect.setProperty(opcodeScroll, "percentContentWidth", 100);

            opcodeBox = new VBox();
            opcodeBox.percentWidth = 100;
            opcodeScroll.addComponent(opcodeBox);

            status = new Label();
            status.text = "Ready";

            left.addComponent(leftTitle);
            left.addComponent(sourceArea);

            right.addComponent(rightTitle);
            right.addComponent(opcodeScroll);
            right.addComponent(status);

            root.addComponent(left);
            root.addComponent(right);

            app.addComponent(root);

            compile();

            sourceArea.onChange = function(_e:UIEvent) {
                compile();
            };

            sourceArea.onClick = function(_e:MouseEvent) {
                syncListToCaret();
            };

        });
        app.start();
    }

    static function defaultSource():String {
        return "// inspector\n" +
            "let x = 0;\n" +
            "print(x);\n";
    }

    static function compile():Void {
        opcodeBox.removeAllComponents();
        opcodeEntries = [];
        opcodeLines = [];
        selectedIndex = -1;
        var source = sourceArea.text;
        try {
            var tokens = Lexer.tokenize(source);
            var ops = Parser.parseWithLines(tokens);
            var idx = 0;
            for (info in ops) {
                var label = new Label();
                var baseText = opcodeLabel(idx, info.op, info.line);
                label.text = baseText;
                label.percentWidth = 100;
                label.userData = { index: idx, baseText: baseText };
                label.onClick = function(_e:MouseEvent) {
                    var data:Dynamic = label.userData;
                    var index = Std.int(data.index);
                    highlightOpcodeIndex(index);
                    selectLine(opcodeLines[index]);
                };
                opcodeBox.addComponent(label);
                opcodeEntries.push(label);
                opcodeLines.push(info.line);
                idx++;
            }
            status.text = "OK (" + ops.length + " ops)";
        } catch (e) {
            status.text = Std.string(e);
        }
    }

    static function opcodeLabel(index:Int, op:OpCode, line:Int):String {
        return "[" + index + "] " + opcodeToString(op) + " (line " + line + ")";
    }

    static function opcodeToString(op:OpCode):String {
        return switch (op) {
            case OConstant(value): "Constant(" + value + ")";
            case OLoadVar(name): "LoadVar(" + name + ")";
            case OStoreVar(name): "StoreVar(" + name + ")";
            case OAdd: "Add";
            case OSubtract: "Subtract";
            case OMultiply: "Multiply";
            case ODivide: "Divide";
            case OModulo: "Modulo";
            case ONegate: "Negate";
            case OEqual: "Equal";
            case ONotEqual: "NotEqual";
            case OLess: "Less";
            case OGreater: "Greater";
            case OLessEqual: "LessEqual";
            case OGreaterEqual: "GreaterEqual";
            case OJump(target): "Jump(" + target + ")";
            case OJumpIfFalse(target): "JumpIfFalse(" + target + ")";
            case OPrint: "Print";
            case OClear: "Clear";
            case OPop: "Pop";
        };
    }

    static function syncListToCaret():Void {
        var caret = getCaretIndex();
        if (caret < 0) {
            return;
        }
        var line = indexToLine(sourceArea.text, caret);
        var idx = findFirstOpcodeLine(line);
        if (idx >= 0) {
            highlightOpcodeIndex(idx);
        }
    }

    static function highlightOpcodeIndex(index:Int):Void {
        if (index < 0 || index >= opcodeEntries.length) {
            return;
        }
        if (selectedIndex >= 0 && selectedIndex < opcodeEntries.length) {
            var prev = opcodeEntries[selectedIndex];
            var prevData:Dynamic = prev.userData;
            prev.text = prevData.baseText;
        }
        var label = opcodeEntries[index];
        var data:Dynamic = label.userData;
        label.text = "> " + data.baseText;
        selectedIndex = index;
    }

    static function findFirstOpcodeLine(line:Int):Int {
        for (i in 0...opcodeLines.length) {
            if (opcodeLines[i] == line) {
                return i;
            }
        }
        return -1;
    }

    static function selectLine(line:Int):Void {
        var lines = sourceArea.text.split("\n");
        if (line < 1 || line > lines.length) {
            return;
        }
        var start = 0;
        for (i in 0...(line - 1)) {
            start += lines[i].length + 1;
        }
        var end = start + lines[line - 1].length;
        sourceArea.focus = true;
        Reflect.setProperty(sourceArea, "selectionStartIndex", start);
        Reflect.setProperty(sourceArea, "selectionEndIndex", end);
        Reflect.setProperty(sourceArea, "caretIndex", end);
        Reflect.setField(sourceArea, "selectionStart", start);
        Reflect.setField(sourceArea, "selectionEnd", end);
        var el = getTextElement();
        if (el != null) {
            var focusFn = Reflect.field(el, "focus");
            if (focusFn != null) {
                Reflect.callMethod(el, focusFn, []);
            }
            var selectFn = Reflect.field(el, "setSelectionRange");
            if (selectFn != null) {
                Reflect.callMethod(el, selectFn, [start, end]);
            }
        }
    }

    static function getCaretIndex():Int {
        var el = getTextElement();
        if (el != null) {
            var sel = Reflect.field(el, "selectionStart");
            if (sel != null) {
                return Std.int(sel);
            }
        }
        var caret = Reflect.getProperty(sourceArea, "caretIndex");
        if (caret == null) {
            caret = Reflect.getProperty(sourceArea, "selectionStartIndex");
        }
        if (caret == null) {
            caret = Reflect.field(sourceArea, "selectionStart");
        }
        if (caret == null) {
            return -1;
        }
        return Std.int(caret);
    }

    static function getTextElement():Dynamic {
        var el = Reflect.getProperty(sourceArea, "element");
        if (el == null) {
            el = Reflect.field(sourceArea, "element");
        }
        return el;
    }

    static function indexToLine(text:String, index:Int):Int {
        var line = 1;
        var max = index;
        if (max > text.length) {
            max = text.length;
        }
        for (i in 0...max) {
            if (text.charCodeAt(i) == 10) {
                line++;
            }
        }
        return line;
    }
}
