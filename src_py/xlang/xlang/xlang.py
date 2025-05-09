import inspect
from typing import Callable, Dict
from xlang.xlang_py import (
    GCSystem,
    VMInt,
    VMFloat,
    VMString,
    VMKeyVal,
    VMNamed,
    VMTuple,
    VMNull,
    VMBytes,
    Lambda,
    WrappedPyFunction,
    VMWrapper,
    VMRange,
)



def wrap_py_function(gc: GCSystem, func: Callable) -> WrappedPyFunction:
    """
    自动解析Python函数参数并创建XLang接口包装

    Args:
        gc: GCSystem实例
        func: 要包装的Python函数

    Returns:
        包装好的WrappedPyFunction实例
    """
    # 获取函数签名
    sig = inspect.signature(func)
    default_args = []

    # 为每个参数创建命名参数
    for param_name, param in sig.parameters.items():
        # 跳过self参数（如果是方法）
        if param_name == "self" and param.kind == param.POSITIONAL_OR_KEYWORD:
            continue

        # 跳过*args和**kwargs类型的参数
        if param.kind == param.VAR_POSITIONAL or param.kind == param.VAR_KEYWORD:
            continue

        # 获取默认值或提供合理的默认值
        if param.default is not param.empty:
            # 使用函数定义中的默认值
            default_value = param.default
        else:
            # 根据类型注解提供合理的默认值
            if param.annotation is not param.empty:
                if param.annotation == int:
                    default_value = 0
                elif param.annotation == float:
                    default_value = 0.0
                elif param.annotation == str:
                    default_value = ""
                elif param.annotation == bool:
                    default_value = False
                elif param.annotation == list or param.annotation == list:
                    default_value = []
                elif param.annotation == dict or param.annotation == Dict:
                    default_value = {}
                else:
                    default_value = None
            else:
                # 如果没有类型注解，默认为None
                default_value = None

        # 创建命名参数并添加到列表
        named_arg = gc.new_named(param_name, default_value)
        default_args.append(named_arg)

    # 创建函数包装器
    wrapped_func = gc.new_pyfunction()
    wrapped_func.wrap(func, gc.new_tuple(default_args))

    return wrapped_func