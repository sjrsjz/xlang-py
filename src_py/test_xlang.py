import sys
import unittest
import os

# 将当前目录添加到 Python 路径，以便导入 xlang_py 模块
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


from xlang_py import (
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
)


class TestXLang(unittest.TestCase):
    def setUp(self):
        self.gc = GCSystem()

    def test_keyval_basic(self):
        """测试基本的键值对创建和访问"""
        key = self.gc.new_int(10)
        value = self.gc.new_string("hello")
        kv = self.gc.new_keyval(key, value)

        # 测试获取键值
        retrieved_key = kv.get_key()
        retrieved_value = kv.get_value()
        self.assertEqual(retrieved_key.get_value(), 10)
        self.assertEqual(retrieved_value.get_value(), "hello")

        # # 测试修改键值
        new_key = self.gc.new_int(20)
        new_value = self.gc.new_string("world")
        kv.set_key(new_key)
        kv.set_value(new_value)

        # 验证修改后的值
        self.assertEqual(kv.get_key().get_value(), 20)
        self.assertEqual(kv.get_value().get_value(), "world")

    def test_keyval_repr(self):
        """测试键值对的字符串表示"""
        key = self.gc.new_int(10)
        value = self.gc.new_string("hello")
        kv = self.gc.new_keyval(key, value)

        # 测试基本表示
        self.assertEqual(repr(kv), 'VMKeyVal(VMInt(10), VMString("hello"))')

    def test_keyval_nested(self):
        """测试嵌套键值对的字符串表示（递归repr）"""
        inner_key = self.gc.new_int(5)
        inner_value = self.gc.new_string("inner")
        inner_kv = self.gc.new_keyval(inner_key, inner_value)

        outer_key = self.gc.new_string("outer")
        outer_kv = self.gc.new_keyval(outer_key, inner_kv)

        # 测试递归表示
        expected = 'VMKeyVal(VMString("outer"), VMKeyVal(VMInt(5), VMString("inner")))'
        self.assertEqual(repr(outer_kv), expected)

    def test_complex_nested_structure(self):
        """测试复杂的嵌套结构"""
        # 创建一个包含多层嵌套的复杂结构
        inner_kv1 = self.gc.new_keyval(self.gc.new_int(1), self.gc.new_string("value1"))
        inner_kv2 = self.gc.new_keyval(
            self.gc.new_string("key2"), self.gc.new_float(2.5)
        )

        # 创建元组
        tuple_obj = self.gc.new_tuple([inner_kv1, inner_kv2])

        # 创建最外层键值对
        root = self.gc.new_keyval(self.gc.new_string("root"), tuple_obj)

        # 验证复杂结构的字符串表示
        expected = 'VMKeyVal(VMString("root"), VMTuple((VMKeyVal(VMInt(1), VMString("value1")), VMKeyVal(VMString("key2"), VMFloat(2.5)))))'
        self.assertEqual(repr(root), expected)

    def test_gc_collection(self):
        """测试垃圾回收"""
        initial_count = self.gc.object_count()

        # 创建一些对象
        key = self.gc.new_int(10)
        value = self.gc.new_string("hello")
        kv = self.gc.new_keyval(key, value)

        # 验证对象计数增加
        self.assertGreater(self.gc.object_count(), initial_count)

        # 删除引用
        del key
        del value
        del kv

        # 触发垃圾回收
        self.gc.collect()

        # 验证对象计数恢复
        self.assertEqual(self.gc.object_count(), initial_count)

    def test_all_types_in_keyval(self):
        """测试所有支持的类型作为键值对的键和值"""
        types = [
            ("int", self.gc.new_int(42)),
            ("float", self.gc.new_float(3.14)),
            ("string", self.gc.new_string("test")),
            ("null", self.gc.new_null()),
            ("bytes", self.gc.new_bytes(b"binary")),
        ]

        for key_name, key_obj in types:
            for val_name, val_obj in types:
                kv = self.gc.new_keyval(key_obj, val_obj)
                # 只要不崩溃就行
                repr_str = repr(kv)
                self.assertIsInstance(repr_str, str)
                self.assertIn(key_name, repr_str.lower())
                self.assertIn(val_name, repr_str.lower())

    def test_from_dict(self):
        """测试从字典创建键值对"""
        py_dict = {
            "A": 1,
            "B": 2.5,
            "C": "hello",
            "D": None,
            "E": b"binary",
            "F": [
                1,
                2,
                {
                    "G": 3.14,
                    "H": "world",
                },
            ],
        }

        kv = self.gc.from_pydict(py_dict)
        self.assertIsInstance(kv, VMTuple)
        self.assertEqual(kv[2].get_value().get_value(), "hello")

    def test_xlang(self):
        xlang_lambda = self.gc.new_lambda()
        xlang_lambda.load(
            code="""
                @required A; 
                @required B;
                A + B
                """,
            default_args=self.gc.new_tuple([]),
        )

        self.assertEqual(
            xlang_lambda(
                kwargs={"A": self.gc.new_int(1), "B": self.gc.new_int(2)}
            ).get_value(),
            3,
        )

    def test_py_function(self):
        def py_func(a, b):
            return a + b

        wrapped_func = self.gc.new_pyfunction()
        wrapped_func.wrap(
            py_func,
            self.gc.new_tuple(
                [
                    self.gc.new_named(self.gc.new_string("a"), self.gc.new_int(0)),
                    self.gc.new_named(self.gc.new_string("b"), self.gc.new_int(0)),
                ]
            ),
        )

        xlang_lambda = self.gc.new_lambda()
        xlang_lambda.load(
            code="""
                @required add; 
                add(1, 2)
                """,
            default_args=self.gc.new_tuple([]),
        )

        self.assertEqual(
            xlang_lambda(
                kwargs={"add": wrapped_func}
            ).get_value(),
            3,
        )

    def __del__(self):
        # 清理资源
        self.gc.collect()
        self.assertEqual(self.gc.object_count(), 0)


if __name__ == "__main__":
    unittest.main()
