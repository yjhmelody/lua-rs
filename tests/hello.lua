    print("hello")

    local a = {
        a = 1,
        b = {
            b = 2,
            c =  {
                c = 3
            }
        }
    }
    print("table")
    print(a.a)
    print(a.b.b)
    print(a.b.c.c)

    print("if")
    b = 10
    if b == 10 then
        print(1)
        print(3)
        b = 20
    elseif b == 20 then
        print(2)
    else
        print(3)
    end

    print(b)

    print("for num")
    for i = 1, 5, 2 do
        print(i)
    end

    print("while")
    i = 1
    while i < 5 do
        i = i + 1
        print(i)
    end

    print("until")
    i = 1
    repeat
        i = i + 1
    print(i)
    until (i > 4)


function bar(x, y)
    print(x, y, z)

    print("until")
    i = 1
    repeat
        i = i + 1
    print(i)
    until (i > 4)
end

bar(1, 2, 3)

do
    print("do")
end


function fibonacii()
    local m = 1
    local n = 1
    while true do
        coroutine.yield(m)
        m, n = n , m + n
    end
end

gen = coroutine.create(fibonacii)
succeeded, value = coroutine.resume(gen)
print(value)
succeeded, value = coroutine.resume(gen)
print(value)
succeeded, value = coroutine.resume(gen)
print(value)
succeeded, value = coroutine.resume(gen)
print(value)
succeeded, value = coroutine.resume(gen)
print(value)
succeeded, value = coroutine.resume(gen)
print(value)
