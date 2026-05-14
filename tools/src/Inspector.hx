package src;

import haxe.ui.HaxeUIApp;
import haxe.ui.components.Label;
import haxe.ui.components.TextArea;
import haxe.ui.containers.HBox;
import haxe.ui.containers.ScrollView;
import haxe.ui.containers.VBox;
import haxe.ui.events.KeyboardEvent;
import haxe.ui.events.UIEvent;
import haxe.ui.events.MouseEvent;
import js.Browser;
import src.Parser.OpCode;

class Inspector {
    static var sourceArea:TextArea;
    static var status:Label;
    static var opcodeScroll:ScrollView;
    static var opcodeBox:VBox;
    static var opcodeEntries:Array<Label> = [];
    static var opcodeLines:Array<Int> = [];
    static var selectedIndex:Int = -1;
    static var syntaxOverlay:Dynamic;
    static var syntaxContent:Dynamic;
    static var overlayInitAttempts:Int = 0;

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
        Browser.document.title = "Rustrial Script Inspector";
        if (Browser.document.body != null) {
            Browser.document.body.style.margin = "0";
            Browser.document.body.style.width = "100vw";
            Browser.document.body.style.height = "100vh";
            Browser.document.body.style.overflow = "hidden";
        }
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

            setupSyntaxOverlay();
            compile();

            sourceArea.onChange = function(_e:UIEvent) {
                compile();
            };

            sourceArea.onClick = function(_e:MouseEvent) {
                syncListToCaret();
            };

            sourceArea.registerEvent(MouseEvent.MOUSE_UP, function(_e:MouseEvent) {
                syncListToCaret();
            });

            sourceArea.registerEvent(KeyboardEvent.KEY_UP, function(_e:KeyboardEvent) {
                syncListToCaret();
            });
        });
        app.start();
    }

    static function defaultSource():String {
        return "// inspector\n" +
            "let x = 0;\n" +
            "print(x);\n";
    }

    static function compile():Void {
        if (syntaxOverlay == null) {
            setupSyntaxOverlay();
        }
        opcodeBox.removeAllComponents();
        opcodeEntries = [];
        opcodeLines = [];
        selectedIndex = -1;
        var source = sourceArea.text;
        updateSyntaxOverlay(source);
        try {
            var tokens = Lexer.tokenize(source);
            var ops = Parser.parseWithLines(tokens);
            var idx = 0;
            for (info in ops) {
                var label = new Label();
                var baseText = opcodeLabel(idx, info.op, info.line);
                label.text = baseText;
                label.percentWidth = 100;
                applyOpcodeStyle(label, info.op);
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
        var text = "[" + index + "] " + opcodeToString(op) + " (line " + line + ")";
        return switch (op) {
            case OJump(target): text + " -> " + target;
            case OJumpIfFalse(target): text + " -> " + target;
            default: text;
        };
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

    static function applyOpcodeStyle(label:Label, op:OpCode):Void {
        var color = opcodeColor(op);
        if (color != null) {
            Reflect.setProperty(label, "styleString", "color: " + color + ";");
        }
    }

    static function opcodeColor(op:OpCode):String {
        return switch (op) {
            case OAdd | OSubtract | OMultiply | ODivide | OModulo | ONegate:
                "#3a89ff";
            case OJump(_) | OJumpIfFalse(_):
                "#ff8a3d";
            case OPrint | OClear:
                "#2fbf71";
            default:
                "#d0d0d0";
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
        var root = Reflect.getProperty(sourceArea, "element");
        if (root == null) {
            root = Reflect.field(sourceArea, "element");
        }
        if (root == null) {
            return null;
        }
        var tag = Reflect.field(root, "tagName");
        if (tag != null && Std.string(tag).toLowerCase() == "textarea") {
            return root;
        }
        var query = Reflect.field(root, "querySelector");
        if (query != null) {
            var ta = Reflect.callMethod(root, query, ["textarea"]);
            if (ta != null) {
                return ta;
            }
        }
        return null;
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

    static function setupSyntaxOverlay():Void {
        if (syntaxOverlay != null) {
            return;
        }
        var el = getTextElement();
        if (el == null) {
            if (overlayInitAttempts < 20) {
                overlayInitAttempts++;
                Browser.window.setTimeout(function() {
                    setupSyntaxOverlay();
                }, 50);
            }
            return;
        }
        var parent = Reflect.field(el, "parentElement");
        if (parent == null) {
            return;
        }
        var overlay = Browser.document.createElement("div");
        var content = Browser.document.createElement("div");
        overlay.appendChild(content);
        syntaxOverlay = overlay;
        syntaxContent = content;

        var parentStyle = Reflect.field(parent, "style");
        if (parentStyle != null) {
            Reflect.setProperty(parentStyle, "position", "relative");
            Reflect.setProperty(parentStyle, "overflow", "hidden");
        }

        var computed = Browser.window.getComputedStyle(el);
        var padding = Reflect.field(computed, "padding");
        var margin = Reflect.field(computed, "margin");
        var border = Reflect.field(computed, "border");

        var overlayStyle = Reflect.field(overlay, "style");
        Reflect.setProperty(overlayStyle, "position", "absolute");
        Reflect.setProperty(overlayStyle, "top", "0");
        Reflect.setProperty(overlayStyle, "left", "0");
        Reflect.setProperty(overlayStyle, "width", "100%");
        Reflect.setProperty(overlayStyle, "height", "100%");
        Reflect.setProperty(overlayStyle, "margin", "0");
        Reflect.setProperty(overlayStyle, "border", "none");
        Reflect.setProperty(overlayStyle, "padding", "0");
        Reflect.setProperty(overlayStyle, "overflow", "hidden");
        Reflect.setProperty(overlayStyle, "pointerEvents", "none");
        Reflect.setProperty(overlayStyle, "zIndex", "-1");
        Reflect.setProperty(overlayStyle, "backgroundColor", "transparent");
        
        var contentStyle = Reflect.field(content, "style");
        Reflect.setProperty(contentStyle, "whiteSpace", "pre-wrap");
        Reflect.setProperty(contentStyle, "wordBreak", "break-word");
        Reflect.setProperty(contentStyle, "boxSizing", "border-box");
        Reflect.setProperty(contentStyle, "margin", "0");
        Reflect.setProperty(contentStyle, "padding", padding);
        Reflect.setProperty(contentStyle, "fontFamily", Reflect.field(computed, "fontFamily"));
        Reflect.setProperty(contentStyle, "fontSize", Reflect.field(computed, "fontSize"));
        Reflect.setProperty(contentStyle, "lineHeight", Reflect.field(computed, "lineHeight"));

        var elStyle = Reflect.field(el, "style");
        Reflect.setProperty(elStyle, "background", "transparent");
        Reflect.setProperty(elStyle, "color", "rgba(0, 0, 0, 0)");
        Reflect.setProperty(elStyle, "textShadow", "none");
        Reflect.setProperty(elStyle, "caretColor", "#111111");
        Reflect.setProperty(elStyle, "position", "relative");
        Reflect.setProperty(elStyle, "zIndex", "0");

        var insertBefore = Reflect.field(parent, "insertBefore");
        if (insertBefore != null) {
            Reflect.callMethod(parent, insertBefore, [overlay, el]);
        }

        var addEvent = Reflect.field(el, "addEventListener");
        if (addEvent != null) {
            Reflect.callMethod(el, addEvent, ["scroll", function(_e) {
                syncSyntaxScroll();
            }]);
        }

        syncSyntaxScroll();
        updateSyntaxOverlay(sourceArea.text);
    }

    static function updateSyntaxOverlay(source:String):Void {
        if (syntaxContent == null) {
            return;
        }
        var html = buildHighlightHtml(source);
        Reflect.setProperty(syntaxContent, "innerHTML", html);
        syncSyntaxScroll();
    }

    static function syncSyntaxScroll():Void {
        if (syntaxContent == null) {
            return;
        }
        var el = getTextElement();
        if (el == null) {
            return;
        }
        var scrollTop = Reflect.field(el, "scrollTop");
        var scrollLeft = Reflect.field(el, "scrollLeft");
        var x = scrollLeft != null ? Std.int(scrollLeft) : 0;
        var y = scrollTop != null ? Std.int(scrollTop) : 0;
        var style = Reflect.field(syntaxContent, "style");
        Reflect.setProperty(style, "transform", "translate(" + (-x) + "px, " + (-y) + "px)");
    }

    static function buildHighlightHtml(source:String):String {
        var buf = new StringBuf();
        var i = 0;
        var len = source.length;

        while (i < len) {
            var code = source.charCodeAt(i);

            if (code == 47 && i + 1 < len && source.charCodeAt(i + 1) == 47) {
                var start = i;
                i += 2;
                while (i < len && source.charCodeAt(i) != 10) {
                    i++;
                }
                buf.add(wrapSpan("#7f7f7f", source.substr(start, i - start)));
                continue;
            }

            if (isDigit(code)) {
                var startNum = i;
                i++;
                while (i < len && isDigit(source.charCodeAt(i))) {
                    i++;
                }
                buf.add(wrapSpan("#3fd3d3", source.substr(startNum, i - startNum)));
                continue;
            }

            if (isAlpha(code)) {
                var startIdent = i;
                i++;
                while (i < len && isAlphaNum(source.charCodeAt(i))) {
                    i++;
                }
                var ident = source.substr(startIdent, i - startIdent);
                if (isKeyword(ident)) {
                    buf.add(wrapSpan("#ff8a3d", ident));
                } else {
                    buf.add(wrapSpan("#e0e0e0", ident));
                }
                continue;
            }

            if (isOperatorStart(code)) {
                var opLen = 1;
                if (i + 1 < len) {
                    var next = source.charCodeAt(i + 1);
                    if ((code == 61 && next == 61) || (code == 33 && next == 61) ||
                        (code == 60 && next == 61) || (code == 62 && next == 61)) {
                        opLen = 2;
                    }
                }
                buf.add(wrapSpan("#ffd24d", source.substr(i, opLen)));
                i += opLen;
                continue;
            }

            buf.add(escapeHtmlChar(source.charAt(i)));
            i++;
        }

        return buf.toString();
    }

    static inline function isDigit(code:Int):Bool {
        return code >= 48 && code <= 57;
    }

    static inline function isAlpha(code:Int):Bool {
        return (code >= 65 && code <= 90) || (code >= 97 && code <= 122) || code == 95;
    }

    static inline function isAlphaNum(code:Int):Bool {
        return isAlpha(code) || isDigit(code);
    }

    static inline function isOperatorStart(code:Int):Bool {
        return code == 43 || code == 45 || code == 42 || code == 47 || code == 37 ||
            code == 61 || code == 33 || code == 60 || code == 62 || code == 40 ||
            code == 41 || code == 123 || code == 125 || code == 59;
    }

    static inline function isKeyword(text:String):Bool {
        return text == "let" || text == "if" || text == "else" ||
            text == "while" || text == "print" || text == "clear";
    }

    static function wrapSpan(color:String, text:String):String {
        return "<span style=\"color: " + color + "\">" + escapeHtml(text) + "</span>";
    }

    static function escapeHtml(text:String):String {
        var buf = new StringBuf();
        for (i in 0...text.length) {
            buf.add(escapeHtmlChar(text.charAt(i)));
        }
        return buf.toString();
    }

    static function escapeHtmlChar(ch:String):String {
        return switch (ch) {
            case "&": "&amp;";
            case "<": "&lt;";
            case ">": "&gt;";
            case "\"": "&quot;";
            case "\t": "    ";
            default: ch;
        };
    }
}
