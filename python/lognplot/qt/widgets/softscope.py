""" A complete softscope widget.

Include this into your application to view signals.
"""

import queue

from ..qtapi import QtWidgets, QtCore
from ...tsdb import TsDb
from .chartwidget import ChartWidget
from .signal_list_widget import SignalListWidget
from .timespan_toolbutton import DurationToolButton


class SoftScope(QtWidgets.QWidget):
    """ A complete softscope widget.

    Include this into your application to view signals.
    """

    def __init__(self):
        super().__init__()
        self.db = TsDb()

        self._last_span = None  # Tail chasing

        # Child widgets:
        self._signal_view = SignalListWidget(self.db)
        self._chart_widget = ChartWidget(self.db)

        toolbar = QtWidgets.QToolBar()
        self._span_selector = DurationToolButton()
        toolbar.addWidget(self._span_selector)

        # Layouting:
        l = QtWidgets.QVBoxLayout()
        # l3 = QtWidgets.QHBoxLayout()
        # l3.addWidget(toolbar)
        l.addWidget(toolbar)
        splitter = QtWidgets.QSplitter()
        splitter.addWidget(self._signal_view)
        splitter.addWidget(self._chart_widget)
        l2 = QtWidgets.QHBoxLayout()
        l2.addWidget(splitter)
        l.addLayout(l2)
        self.setLayout(l)

        # Connect signals:
        self._chart_widget.manually_zoomed.connect(self._disable_tailing)
        self._span_selector.duration_selected.connect(self._enable_tailing)

        self._rx_queue = queue.Queue()
        self._timer = QtCore.QTimer()
        self._timer.timeout.connect(self._on_timeout)
        self._timer.start(10)

    def add_samples(self, channel, samples):
        """ Call this function with new data.

        Data is stored into a queue, so this is presumably thread safe.
        """
        self._rx_queue.put((channel, samples))

    def add_curve(self, name, color):
        self._chart_widget.add_curve(name, color)

    def _enable_tailing(self, timespan):
        """ Slot to enable tailing the last timespan of the signals. """
        self._last_span = timespan

    def _disable_tailing(self):
        """ Stop tailing the signals in view. """
        self._last_span = None

    def _on_timeout(self):
        if not self._rx_queue.empty():
            while not self._rx_queue.empty():
                chunk = self._rx_queue.get()
                name, samples = chunk
                self.db.add_samples(name, samples)
                self._rx_queue.task_done()
            self._chart_widget.update()

        # Follow last x seconds:
        if self._last_span:
            self._chart_widget.zoom_to_last(self._last_span)

        # Hmm, ugly polling?
        self._signal_view._signal_list_model.update()
