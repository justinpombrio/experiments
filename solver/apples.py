equations = [
    ["lemon"],
    ["banana", "lemon"],
    ["broc", "peach"],
    ["eggplant", "grape"],
    ["apple", "av", "mango"],
    ["kiwi", "pumpkin", "tomato"],
    ["lemon", "lemon", "peach"],
    ["lemon", "mango", "pumpkin"],
    ["mango", "mango", "pumpkin"],
    ["apple", "kiwi", "mango", "mango"],
    ["av", "blueberry", "mango", "mango", "pumpkin"],
    ["blueberry", "blueberry", "blueberry", "blueberry", "grape"],
    ["kiwi", "kiwi", "lemon", "pumpkin", "tomato"],
    ["pumpkin", "pumpkin", "pumpkin", "pumpkin", "orange"],
    ["blueberry", "blueberry", "blueberry", "blueberry", "grape", "peach", "peach"],
    ["apple", "eggplant", "eggplant", "kiwi", "lemon", "tomato", "tomato", "tomato"],
    ["bean", "blueberry", "broc", "broc", "broc", "broc", "grape", "grape", "orange"],
    ["bean", "bean", "broc", "eggplant", "grape", "grape", "grape", "grape", "grape", "grape", "grape"]
]

init_fruits = {
    "lemon": [3617],
    "banana": [4236],
    "broc": [3082],
    "peach": [4399],
    "eggplant": [4602],
    "grape": [3451],
    "apple": [n for n in range(5000)],
    "av": [3080,3354,3509,4046,4221,4222,4223,4224,4225,4226,4227,4228,4770,4771],
    "mango": [3363],
    #"kiwi": [3279],
    "kiwi": [3279,3280,3517,4030,4301],
    #"pumpkin": [3131],
    "pumpkin": [3131, 4735], # 3131
    #"orange": [3155],
    "orange": [3155, 3153],
    "tomato":
    [3061,3145,3146,3147,3148,3149,3150,3151,3282,3335,3336,3423,3458,3512,4063,4064,4087,4664,4778,4796,4797,4798,4799,4800,4801,4802,4803,4804,4805,4806,4807,4808],
    "blueberry": [4240],
    "bean": [4528],
}

def isprime(n):
    return all( n%i != 0 for i in range(2, int(n**.5)+1))

def has_solution(equation, n, fruits):
    if len(equation) == 0:
        return isprime(n)
    fruit = equation[0]
    for val in fruits[fruit]:
        if has_solution(equation[1:], n + val, fruits):
            return True
    return False

for fruit in init_fruits:
    if len(init_fruits[fruit]) == 1: continue
    if fruit == "apple": continue
    valid = []
    for val in init_fruits[fruit]:
        fruits = init_fruits.copy()
        fruits[fruit] = [val]
        is_consistent = True
        for equation in equations:
            if not has_solution(equation, 0, fruits):
                is_consistent = False
        if is_consistent:
            valid.append(val)
    print(fruit, valid)


counts = {}
for eq in equations:
    for fruit in eq:
        if fruit not in counts:
            counts[fruit] = 0
        counts[fruit] += 1
print(counts)
