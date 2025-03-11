export type ValueType = 'uint' | 'string' | 'bool' | 'map' | 'int' | 'float' | 'list' | 'function' | 'bytes' | 'timestamp' | 'null';

export interface PassableValue {
    type: ValueType;
    value: any;
}

export interface PassableMap {
    map: {
        [key: string]: PassableValue;
    };
}

export interface ExecutionContext {
    variables: PassableMap;
    expression: string;
    computed?: { [key: string]: PassableValue[] };
    device?: { [key: string]: PassableValue[] };
}

// Type guards and utility types for specific value types
export interface StringValue extends PassableValue {
    type: 'string';
    value: string;
}

export interface IntValue extends PassableValue {
    type: 'int';
    value: number;
}

export interface UIntValue extends PassableValue {
    type: 'uint';
    value: number;
}

export interface FloatValue extends PassableValue {
    type: 'float';
    value: number;
}

export interface BoolValue extends PassableValue {
    type: 'bool';
    value: boolean;
}

export interface ListValue extends PassableValue {
    type: 'list';
    value: PassableValue[];
}

export interface MapValue extends PassableValue {
    type: 'map';
    value: { [key: string]: PassableValue };
}

export interface BytesValue extends PassableValue {
    type: 'bytes';
    value: number[]; // Represents byte array
}

export interface TimestampValue extends PassableValue {
    type: 'timestamp';
    value: number;
}

export interface FunctionValue extends PassableValue {
    type: 'function';
    value: [string, PassableValue | null]; // Tuple of name and optional argument
}

export interface NullValue extends PassableValue {
    type: 'null';
    value: null;
}

// Type guard functions
export const isStringValue = (value: PassableValue): value is StringValue => value.type === 'string';
export const isIntValue = (value: PassableValue): value is IntValue => value.type === 'int';
export const isUIntValue = (value: PassableValue): value is UIntValue => value.type === 'uint';
export const isFloatValue = (value: PassableValue): value is FloatValue => value.type === 'float';
export const isBoolValue = (value: PassableValue): value is BoolValue => value.type === 'bool';
export const isListValue = (value: PassableValue): value is ListValue => value.type === 'list';
export const isMapValue = (value: PassableValue): value is MapValue => value.type === 'map';
export const isBytesValue = (value: PassableValue): value is BytesValue => value.type === 'bytes';
export const isTimestampValue = (value: PassableValue): value is TimestampValue => value.type === 'timestamp';
export const isFunctionValue = (value: PassableValue): value is FunctionValue => value.type === 'function';
export const isNullValue = (value: PassableValue): value is NullValue => value.type === 'null';

// Host context and module interfaces
export interface SuperscriptHostContext {
    computed_property(name: string, args: [PassableValue]): PassableValue;
    device_property(name: string, args: [PassableValue]): PassableValue;
}

export interface WasmModule {
    evaluate_with_context(input: ExecutionContext, context: SuperscriptHostContext): Promise<boolean>;
} 