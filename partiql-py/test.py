import partiql_py

(res, deser_res) = partiql_py.eval("SELECT * FROM a", "{'a': {'b': 1}}")

print(res)
print(deser_res)