function test_json_deser(json, expected)
    @test deser_json(typeof(expected), json) == expected
end
