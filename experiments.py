import os
import math
import pickle
import pprint

traces = [
    'fp_1.bz2',
    'fp_2.bz2',
    'int_1.bz2',
    'int_2.bz2',
    'mm_1.bz2',
    'mm_2.bz2'
]

optimal_perceptron = {
    1: 12,
    2: 22,
    4: 28,
    8: 34,
    16: 36,
    32: 59,
    64: 59,
    128: 62,
    256: 62,
    512: 62
}

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

def calc_theta(h):
    return round(1.93 * h + 14)

def calc_n_percep(budget, theta, h):
    return round((budget*1024*8-h)/((math.log2(theta)+1)*h))
    # return round(budget/(theta*h))


if __name__ == "__main__":
    # gshare uses 13 bits (see readme)
    gshare_conf = {'lh_bits': 13}
    # tournament has 9 global history, 10 local, and 10 PC index bits
    tournament_conf = {'gh_bits': 9, 'lh_bits': 10, 'index': 10}
    # best values based on an 8KiB (64Kib) budget from the paper
    perceptron_conf = {'h_bits': 34, 'n_percep': 263, 'theta': 79}

    results = {}

    for name_trace in traces:
        # Results for the optimal config under a certain size = ?
        gshare_miss = run_gshare_predictor(name_trace, gshare_conf['lh_bits'])
        tourn_miss = run_tournament_predictor(name_trace, tournament_conf['gh_bits'], tournament_conf['lh_bits'], tournament_conf['index'])
        custom_miss = run_custom_predictor(name_trace, perceptron_conf['h_bits'], perceptron_conf['n_percep'], perceptron_conf['theta'])

        # Results when varying the size

        custom_vary = []
        for budget, h_bits in optimal_perceptron.items():
            theta = calc_theta(h_bits)
            custom_miss = run_custom_predictor(name_trace, h_bits, calc_n_percep(budget, theta, h_bits), theta)
            custom_vary.append(custom_miss)

        gshare_vary = []
        for i in range(9,19):
            gshare_miss = run_gshare_predictor(name_trace, i)
            gshare_vary.append(gshare_miss)

        tourn_vary = []
        for i in range(8,18):
            tourn_miss = run_tournament_predictor(name_trace, i+1, i+1, i)
            tourn_vary.append(tourn_miss)

        trace_res = {
            'gshare': gshare_miss,
            'tourn': tourn_miss,
            'custom': custom_miss,
            'custom_vary': custom_vary,
            'gshare_vary': gshare_vary,
            'tourn_vary': tourn_vary
        }

        results[name_trace] = trace_res

    results_file = open("results_exp.pkl", "wb")
    pickle.dump(results, results_file)
    results_file.close()


    pprint.pprint(results, width=20)


