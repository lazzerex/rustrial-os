package tools.src;

import tools.src.Lexer.Token;
import tools.src.Lexer.TokenInfo;
import Type;

enum OpCode {
    OConstant(value:Int);
    OLoadVar(name:String);
    OStoreVar(name:String);

    OAdd;
    OSubtract;
    OMultiply;
    ODivide;
    OModulo;
    ONegate;

    OEqual;
    ONotEqual;
    OLess;
    OGreater;
    OLessEqual;
    OGreaterEqual;

    OJump(target:Int);
    OJumpIfFalse(target:Int);

    OPrint;
    OClear;

    OPop;
}

typedef OpCodeInfo = {
    var op: OpCode;
    var line: Int;
}

class Parser {
    var tokens:Array<TokenInfo>;
    var current:Int;
    var bytecode:Array<OpCode>;
    var lineMap:Array<Int>;
    var lastLine:Int;

    function new(tokens:Array<TokenInfo>) {
        this.tokens = tokens;
        this.current = 0;
        this.bytecode = [];
        this.lineMap = [];
        this.lastLine = tokens.length > 0 ? tokens[0].line : 1;
    }

    public static function parse(tokens:Array<TokenInfo>):Array<OpCode> {
        var parser = new Parser(tokens);
        parser.parseProgram();
        return parser.bytecode;
    }

    public static function parseWithLines(tokens:Array<TokenInfo>):Array<OpCodeInfo> {
        var parser = new Parser(tokens);
        parser.parseProgram();
        return parser.buildOpInfo();
    }

    inline function isAtEnd():Bool {
        return switch (peekKind()) {
            case TEof: true;
            default: false;
        };
    }

    inline function peek():TokenInfo {
        return tokens[current];
    }

    inline function peekKind():Token {
        return tokens[current].kind;
    }

    function advance():TokenInfo {
        if (!isAtEnd()) {
            current++;
        }
        var token = tokens[current - 1];
        lastLine = token.line;
        return token;
    }

    inline function sameKind(a:Token, b:Token):Bool {
        return Type.enumIndex(a) == Type.enumIndex(b);
    }

    function check(expected:Token):Bool {
        if (isAtEnd()) {
            return false;
        }
        return sameKind(peekKind(), expected);
    }

    function consume(expected:Token, msg:String):Void {
        if (check(expected)) {
            advance();
            return;
        }
        fail(peek().line, msg);
    }

    inline function fail(line:Int, msg:String):Void {
        throw "Line " + line + ": " + msg;
    }

    inline function emit(op:OpCode):Void {
        bytecode.push(op);
        lineMap.push(lastLine);
    }

    function buildOpInfo():Array<OpCodeInfo> {
        var result = new Array<OpCodeInfo>();
        for (i in 0...bytecode.length) {
            var line = i < lineMap.length ? lineMap[i] : lastLine;
            result.push({ op: bytecode[i], line: line });
        }
        return result;
    }

    function parseProgram():Void {
        while (!isAtEnd()) {
            parseStatement();
        }
    }

    function parseStatement():Void {
        switch (peekKind()) {
            case TLet: parseLet();
            case TIf: parseIf();
            case TWhile: parseWhile();
            case TPrint: parsePrint();
            case TClear: parseClear();
            case TLeftBrace: parseBlock();
            case TIdentifier(_): parseAssignmentOrExpr();
            default:
                parseExpression();
                consume(TSemicolon, "Expected ';'");
                emit(OPop);
        }
    }

    function parseLet():Void {
        advance();

        var token = advance();
        var name = switch (token.kind) {
            case TIdentifier(n): n;
            default:
                fail(token.line, "Expected identifier after 'let'");
                "";
        };

        consume(TEqual, "Expected '=' after variable name");
        parseExpression();
        consume(TSemicolon, "Expected ';'");

        emit(OStoreVar(name));
    }

    function parseAssignmentOrExpr():Void {
        var token = advance();
        var name = switch (token.kind) {
            case TIdentifier(n): n;
            default:
                fail(token.line, "Expected identifier");
                "";
        };

        if (check(TEqual)) {
            advance();
            parseExpression();
            consume(TSemicolon, "Expected ';'");
            emit(OStoreVar(name));
        } else {
            current -= 1;
            parseExpression();
            consume(TSemicolon, "Expected ';'");
            emit(OPop);
        }
    }

    function parseIf():Void {
        advance();

        consume(TLeftParen, "Expected '(' after 'if'");
        parseExpression();
        consume(TRightParen, "Expected ')' after condition");

        var jumpIfFalseIdx = bytecode.length;
        emit(OJumpIfFalse(0));

        parseStatement();

        if (check(TElse)) {
            advance();

            var jumpIdx = bytecode.length;
            emit(OJump(0));

            var elseStart = bytecode.length;
            bytecode[jumpIfFalseIdx] = OJumpIfFalse(elseStart);

            parseStatement();

            var afterElse = bytecode.length;
            bytecode[jumpIdx] = OJump(afterElse);
        } else {
            var afterIf = bytecode.length;
            bytecode[jumpIfFalseIdx] = OJumpIfFalse(afterIf);
        }
    }

    function parseWhile():Void {
        advance();

        var loopStart = bytecode.length;

        consume(TLeftParen, "Expected '(' after 'while'");
        parseExpression();
        consume(TRightParen, "Expected ')' after condition");

        var jumpIfFalseIdx = bytecode.length;
        emit(OJumpIfFalse(0));

        parseStatement();

        emit(OJump(loopStart));

        var afterLoop = bytecode.length;
        bytecode[jumpIfFalseIdx] = OJumpIfFalse(afterLoop);
    }

    function parsePrint():Void {
        advance();

        consume(TLeftParen, "Expected '(' after 'print'");
        parseExpression();
        consume(TRightParen, "Expected ')' after expression");
        consume(TSemicolon, "Expected ';'");

        emit(OPrint);
    }

    function parseClear():Void {
        advance();

        consume(TLeftParen, "Expected '(' after 'clear'");
        consume(TRightParen, "Expected ')'");
        consume(TSemicolon, "Expected ';'");

        emit(OClear);
    }

    function parseBlock():Void {
        advance();

        while (!check(TRightBrace) && !isAtEnd()) {
            parseStatement();
        }

        consume(TRightBrace, "Expected '}'");
    }

    function parseExpression():Void {
        parseComparison();
    }

    function parseComparison():Void {
        parseTerm();

        while (isComparisonOp(peekKind())) {
            var op = advance().kind;
            parseTerm();

            switch (op) {
                case TEqualEqual: emit(OEqual);
                case TBangEqual: emit(ONotEqual);
                case TLess: emit(OLess);
                case TGreater: emit(OGreater);
                case TLessEqual: emit(OLessEqual);
                case TGreaterEqual: emit(OGreaterEqual);
                default:
                    fail(peek().line, "Unexpected comparison operator");
            }
        }
    }

    function parseTerm():Void {
        parseFactor();

        while (isTermOp(peekKind())) {
            var op = advance().kind;
            parseFactor();

            switch (op) {
                case TPlus: emit(OAdd);
                case TMinus: emit(OSubtract);
                default:
                    fail(peek().line, "Unexpected term operator");
            }
        }
    }

    function parseFactor():Void {
        parseUnary();

        while (isFactorOp(peekKind())) {
            var op = advance().kind;
            parseUnary();

            switch (op) {
                case TStar: emit(OMultiply);
                case TSlash: emit(ODivide);
                case TPercent: emit(OModulo);
                default:
                    fail(peek().line, "Unexpected factor operator");
            }
        }
    }

    function parseUnary():Void {
        if (check(TMinus)) {
            advance();
            parseUnary();
            emit(ONegate);
        } else {
            parsePrimary();
        }
    }

    function parsePrimary():Void {
        var token = advance();
        switch (token.kind) {
            case TNumber(n):
                emit(OConstant(n));
            case TIdentifier(name):
                emit(OLoadVar(name));
            case TLeftParen:
                parseExpression();
                consume(TRightParen, "Expected ')' after expression");
            default:
                fail(token.line, "Unexpected token in expression");
        }
    }

    inline function isComparisonOp(token:Token):Bool {
        return switch (token) {
            case TEqualEqual | TBangEqual | TLess | TGreater | TLessEqual | TGreaterEqual: true;
            default: false;
        };
    }

    inline function isTermOp(token:Token):Bool {
        return switch (token) {
            case TPlus | TMinus: true;
            default: false;
        };
    }

    inline function isFactorOp(token:Token):Bool {
        return switch (token) {
            case TStar | TSlash | TPercent: true;
            default: false;
        };
    }
}
