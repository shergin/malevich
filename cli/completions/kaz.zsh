#compdef kaz
# zsh completion for kaz. Hand-written; the packaging test guards it against
# drift with the subcommand table.
#
# Install: place this file (named `_kaz`) on your $fpath.

_kaz() {
  _arguments -C \
    '-o[plot destination: - for stdout, or FILE]:target:_files' \
    '-O[pass input through to stdout]' \
    '-d[field separator]:char:' \
    '-H[first row is a header]' \
    '--fmt[column mapping]:fmt:(y xy xyy xyxy yx)' \
    '-w[width in cells]:cells:' \
    '-h[height in cells]:cells:' \
    '-t[title]:title:' \
    '--xlabel[x-axis title]:text:' \
    '--ylabel[y-axis title]:text:' \
    '--xlim[fix x range]:A,B:' \
    '--ylim[fix y range]:A,B:' \
    '--log-x[log-scale x]' \
    '--log-y[log-scale y]' \
    '--time-x[read x as time]' \
    '--bins[histogram bin count]:n:' \
    '--colormap[heatmap/hist2d colors]:map:(viridis magma cividis greys red-blue purple-orange)' \
    '--midpoint[center the colormap on a value]:v:' \
    '--log-color[logarithmic colormap]' \
    '--labels-x[heatmap band labels across the columns]:labels:' \
    '--labels-y[heatmap band labels down the rows]:labels:' \
    '--reduce[dense-heatmap bucket summary]:reducer:(mean max min median)' \
    '--cols[select and reorder columns]:list:' \
    '--by[scatter: color by this column]:col:' \
    '--emit-code[print the equivalent malevich program]' \
    '--color[when to color]:when:(auto always never)' \
    '--charset[glyph tier]:set:(auto ascii half quad sextant braille octant)' \
    '--pixels[pixel image panel]:when:(auto always never)' \
    '-q[suppress the unparsed tally]' \
    '--live[stream stdin, repaint in place]' \
    '--window[live window length]:n:' \
    '--fps[live repaint rate]:n:' \
    '--rate[plot counter deltas]' \
    '--version[print version]' \
    '--help[show help]' \
    '1:chart:(line scatter bar hist count density ecdf box violin hist2d heatmap)' \
    '*:file:_files'
}

_kaz "$@"
