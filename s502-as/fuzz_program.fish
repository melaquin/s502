#!/usr/bin/env fish

mkdir -p afl_out

screen -dmS fuzzer15 cargo afl fuzz -i test_inputs -o afl_out -b 15 -M fuzzer15 ../target/debug/s502-as -b

for core in (seq 6 14)
    # TODO master and 4 others here have -b, others don't
    # so if core is even then pass -b
    set -l binary -b
    if test (math $core % 2) -eq 1
        set -e binary
    end
    screen -dmS fuzzer$core cargo afl fuzz -i test_inputs -o afl_out -b $core -S fuzzer$core ../target/debug/s502-as $binary
end

read -P "Press enter to stop fuzzing... "

for core in (seq 6 15)
    screen -S fuzzer$core -X quit
end