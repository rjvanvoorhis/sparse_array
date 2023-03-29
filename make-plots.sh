#RANK
python3 make_plots.py runs/rank-support-dynamic-block-size -y query_duration --title "Rank Support: query time vs. length" -o plots/rank-time.png
python3 make_plots.py runs/rank-support-dynamic-block-size -y overhead --title "Rank Support: overhead vs. length" -o plots/rank-overhead.png

# SELECT
python3 make_plots.py runs/select-support-dynamic-block-size -y query_duration --title "Select Support: query time vs. length" -o plots/select-log-time.png --logx=true
python3 make_plots.py runs/select-support-dynamic-block-size -y query_duration --title "Select Support: query time vs. length" -o plots/select-time.png
python3 make_plots.py runs/select-support-dynamic-block-size -y overhead --title "Select Support: overhead vs. length" -o plots/select-overhead.png

#SPARSE
python3 make_plots.py runs/sparse-array-get-at-index-vary-by-length -y query_duration --title "Sparse Array (15% populated): get_at_index query time vs. length" -o plots/sparse-get-at-index-time-v-length.png
python3 make_plots.py runs/sparse-array-get-at-index-vary-by-sparsity -y query_duration --xlabel="Percent populated" --title "Sparse Array (500k length): get_at_index query time vs. sparsity" -o plots/sparse-get-at-index-time-v-sparsity.png

python3 make_plots.py runs/sparse-array-get-index-of-vary-by-length -y query_duration --title "Sparse Array (15% populated): get_index_of query time vs. length" -o plots/sparse-get-index-of-time-v-length.png
python3 make_plots.py runs/sparse-array-get-index-of-vary-by-sparsity -y query_duration --xlabel="Percent populated" --title "Sparse Array (500k length): get_index_of query time vs. sparsity" -o plots/sparse-get-index-of-time-v-sparsity.png

python3 make_plots.py runs/sparse-array-num-elem-at-vary-by-length -y query_duration --title "Sparse Array (15% populated): num_elem_at query time vs. length" -o plots/sparse-num-elem-at-index-time-v-length.png
python3 make_plots.py runs/sparse-array-num-elem-at-vary-by-sparsity -y query_duration --xlabel="Percent populated" --title "Sparse Array (500k length): num_elem_at query time vs. sparsity" -o plots/sparse-num-elem-at-time-v-sparsity.png