package tools.src;

typedef TokenInfo = {
    var kind: Token;
    var line: Int;
}

enum Token {
    TNumber(value:Int);
    TIdentifier(value:String);

    TLet;
    TIf;
    TElse;
    TWhile;
    TPrint;
    TClear;

    TPlus;
    TMinus;
    TStar;
    TSlash;
    TPercent;
    TEqual;
    TEqualEqual;
    TBangEqual;
    TLess;
    TGreater;
    TLessEqual;
    TGreaterEqual;

    TLeftParen;
    TRightParen;
    TLeftBrace;
    TRightBrace;
    TSemicolon;

    TEof;
}

class Lexer {
    public static function tokenize(source:String):Array<TokenInfo> {
        var tokens = new Array<TokenInfo>();
        var i = 0;
        var line = 1;
        var len = source.length;

        inline function add(kind:Token):Void {
            tokens.push({ kind: kind, line: line });
        }

        inline function codeAt(idx:Int):Int {
            return source.charCodeAt(idx);
        }

        inline function isDigit(code:Int):Bool {
            return code >= 48 && code <= 57;
        }

        inline function isAlpha(code:Int):Bool {
            return (code >= 65 && code <= 90) || (code >= 97 && code <= 122) || code == 95;
        }

        inline function isAlphaNum(code:Int):Bool {
            return isAlpha(code) || isDigit(code);
        }

        while (i < len) {
            var code = codeAt(i);

            if (code == 32 || code == 9 || code == 13) {
                i++;
                continue;
            }

            if (code == 10) {
                line++;
                i++;
                continue;
            }

            if (isDigit(code)) {
                var num = 0;
                while (i < len && isDigit(codeAt(i))) {
                    var digit = codeAt(i) - 48;
                    num = (num * 10 + digit) | 0;
                    i++;
                }
                add(TNumber(num));
                continue;
            }

            if (isAlpha(code)) {
                var start = i;
                i++;
                while (i < len && isAlphaNum(codeAt(i))) {
                    i++;
                }
                var ident = source.substr(start, i - start);
                switch (ident) {
                    case "let": add(TLet);
                    case "if": add(TIf);
                    case "else": add(TElse);
                    case "while": add(TWhile);
                    case "print": add(TPrint);
                    case "clear": add(TClear);
                    default: add(TIdentifier(ident));
                }
                continue;
            }

            switch (code) {
                case 43: // +
                    add(TPlus);
                    i++;
                case 45: // -
                    add(TMinus);
                    i++;
                case 42: // *
                    add(TStar);
                    i++;
                case 47: // /
                    if (i + 1 < len && codeAt(i + 1) == 47) {
                        i += 2;
                        while (i < len) {
                            var c = codeAt(i);
                            if (c == 10) {
                                line++;
                                i++;
                                break;
                            }
                            i++;
                        }
                    } else {
                        add(TSlash);
                        i++;
                    }
                case 37: // %
                    add(TPercent);
                    i++;
                case 61: // =
                    if (i + 1 < len && codeAt(i + 1) == 61) {
                        add(TEqualEqual);
                        i += 2;
                    } else {
                        add(TEqual);
                        i++;
                    }
                case 33: // !
                    if (i + 1 < len && codeAt(i + 1) == 61) {
                        add(TBangEqual);
                        i += 2;
                    } else {
                        throw "Unexpected character '!'";
                    }
                case 60: // <
                    if (i + 1 < len && codeAt(i + 1) == 61) {
                        add(TLessEqual);
                        i += 2;
                    } else {
                        add(TLess);
                        i++;
                    }
                case 62: // >
                    if (i + 1 < len && codeAt(i + 1) == 61) {
                        add(TGreaterEqual);
                        i += 2;
                    } else {
                        add(TGreater);
                        i++;
                    }
                case 40: // (
                    add(TLeftParen);
                    i++;
                case 41: // )
                    add(TRightParen);
                    i++;
                case 123: // {
                    add(TLeftBrace);
                    i++;
                case 125: // }
                    add(TRightBrace);
                    i++;
                case 59: // ;
                    add(TSemicolon);
                    i++;
                default:
                    throw "Unexpected character";
            }
        }

        add(TEof);
        return tokens;
    }
}
