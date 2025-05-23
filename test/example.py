from xlang import GCSystem, wrap_py_function, XlangExecutionError

gc = GCSystem()

context = {}


def xlang_print(values):
    print(f"XLang Print:{values}, {repr(values)}")


def xlang_set(**kwargs):
    context.update(kwargs)


def xlang_get():
    return gc.new_tuple([gc.new_named(k, v) for k, v in context.items()])


wrapped_print = wrap_py_function(gc, xlang_print)
wrapped_set = wrap_py_function(gc, xlang_set)
wrapped_get = wrap_py_function(gc, xlang_get)

import time

last_time = time.time()


def run_condition():
    global last_time
    if time.time() - last_time > 1:
        raise RuntimeError("Timeout")


xlang_lambda = gc.new_lambda()
xlang_lambda.load(
    code="""
        @required print;
        @required let;
        @required context;
        print("Hello from XLang!");

        #let A => 1;
        print(context().A);
        while true {}
        """,
    default_args=gc.new_tuple(
        [
            gc.new_named("print", wrapped_print),
            gc.new_named("let", wrapped_set),
            gc.new_named("context", wrapped_get),
        ],
    ),
    run_condition=run_condition,
)
try:
    print(xlang_lambda())
    print("XLang Lambda executed successfully")
except XlangExecutionError as e:
    print(f"XLang Execution Error: {e}")
finally:
    print("XLang Lambda finished")
del context
del wrapped_print
del wrapped_set
del wrapped_get
del xlang_lambda
gc.collect()
assert gc.object_count() == 0, "GC should have collected all objects"
