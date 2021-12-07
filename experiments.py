import os
import pprint

traces = [
    'fp_1.bz2',
    'fp_2.bz2',
    'int_1.bz2',
    'int_2.bz2',
    'mm_1.bz2',
    'mm_2.bz2'
]

def parse_mispredicition_rate(result):
    #TODO handle error if needed
    data = result.split(':')
    # select last element since it is the percentage and remove the first \t and the newline and the % sign at the end
    return float(data[-1][1:-2])

def run_gshare_predictor(trace, history_bits, only_miss=True):
    res = os.popen('bunzip2 -kc ./traces/{trace} | cargo run --release -- --predictor gshare:{history_bits}'
    .format(
        trace = trace,
        history_bits = str(history_bits)
        )
    ).read()
    if only_miss:
        return parse_mispredicition_rate(res)
    return res

def run_tournament_predictor(trace, ghistory, lhistory, index, only_miss=True):
    res = os.popen('bunzip2 -kc ./traces/{trace} | cargo run --release -- --predictor tournament:{ghistory}:{lhistory}:{index}'
    .format(
        trace = trace,
        ghistory = str(ghistory),
        lhistory = str(lhistory),
        index = str(index)
        )
    ).read()
    if only_miss:
        return parse_mispredicition_rate(res)
    return res

def run_custom_predictor(trace, history_size, num_perceptrons, theta, only_miss=True):
    res = os.popen('bunzip2 -kc ./traces/{trace} | cargo run --release -- --predictor custom:{history_size}:{num_perceptrons}:{theta}'
    .format(
        trace = trace,
        history_size= str(history_size),
        num_perceptrons = str(num_perceptrons),
        theta = str(theta)
        )
    ).read()
    if only_miss:
        return parse_mispredicition_rate(res)
    return res


if __name__ == "__main__":
    # gshare uses 13 bits (see readme)
    gshare_conf = {'lh_bits': 13}
    # tournament has 9 global history, 10 local, and 10 PC index bits
    tournament_conf = {'gh_bits': 9, 'lh_bits': 10, 'index': 10}
    # best values based on an 8KiB (64Kib) budget from the paper
    perceptron_conf = {'h_bits': 34, 'n_percep': 305, 'theta': 79}

    results = {}
    for name_trace in traces:
        gshare_miss = run_gshare_predictor(name_trace, gshare_conf['lh_bits'])
        tourn_miss = run_tournament_predictor(name_trace, tournament_conf['gh_bits'], tournament_conf['lh_bits'], tournament_conf['index'])
        custom_miss = run_custom_predictor(name_trace, perceptron_conf['h_bits'], perceptron_conf['n_percep'], perceptron_conf['theta'])

        trace_res = {
            'gshare': gshare_miss,
            'tourn': tourn_miss,
            'custom': custom_miss
        }
        results[name_trace] = trace_res

    pprint.pprint(results, width=20)
